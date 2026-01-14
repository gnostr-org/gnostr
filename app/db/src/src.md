# src Code Documentation

**Generated on:** 2026-01-13 15:20:01
**Directory:** /Users/Shared/gnostr-org/.github/gnostr/app/db/src
**Files included:** 6

---

## Directory Structure

```
./db.rs
./error.rs
./event.rs
./filter.rs
./key.rs
./lib.rs
./src.md
```

---

## File Contents

### db.rs

**Size:** 35119 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use crate::{
    error::Error,
    key::{concat, concat_sep, encode_replace_key, u16_to_ver, u64_to_ver, IndexKey},
    ArchivedEventIndex, Event, EventIndex, Filter, FromEventData, Stats,
};
use nostr_kv::{
    lmdb::{Db as Lmdb, Iter as LmdbIter, *},
    scanner::{Group, GroupItem, MatchResult, Scanner},
};

use std::{
    marker::PhantomData,
    ops::Bound,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

type Result<T, E = Error> = core::result::Result<T, E>;

pub fn upper(mut key: Vec<u8>) -> Option<Vec<u8>> {
    key.iter().rposition(|&x| x < u8::MAX).map(|position| {
        key[position] += 1;
        key.truncate(position + 1);
        key
    })
}

const MAX_TAG_VALUE_SIZE: usize = 255;
const DB_VERSION: &str = "3";

#[derive(Clone)]
pub struct Db {
    inner: Lmdb,
    #[allow(unused)]
    // save meta data
    t_meta: Tree,
    // save data
    t_data: Tree,
    // save index
    t_index: Tree,
    // map id to uid
    t_id_uid: Tree,
    // map id to word
    t_uid_word: Tree,
    // id time
    t_id: Tree,
    // pubkey time
    t_pubkey: Tree,
    // kind time
    t_kind: Tree,
    t_pubkey_kind: Tree,
    t_created_at: Tree,
    t_tag: Tree,
    t_deletion: Tree,
    t_replacement: Tree,
    t_expiration: Tree,
    // word time
    t_word: Tree,
    seq: Arc<AtomicU64>,
}

fn u64_from_bytes(bytes: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(bytes.try_into()?))
}

fn u16_from_bytes(bytes: &[u8]) -> Result<u16, Error> {
    Ok(u16::from_be_bytes(bytes.try_into()?))
}

// Get the latest seq from db
fn latest_seq(db: &Lmdb, tree: &Tree) -> Result<u64, Error> {
    let txn = db.reader()?;
    let mut iter = txn.iter_from(tree, Bound::Unbounded::<Vec<u8>>, true);
    if let Some(item) = iter.next() {
        let (k, _) = item?;
        u64_from_bytes(k)
    } else {
        Ok(0)
    }
}

#[cfg(feature = "zstd")]
fn encode_event(event: &Event) -> Result<Vec<u8>> {
    let json = event.to_json()?;
    let mut json = zstd::encode_all(json.as_bytes(), 5).map_err(Error::Io)?;
    json.push(1);
    Ok(json)
}
#[cfg(not(feature = "zstd"))]
fn encode_event(event: &Event) -> Result<String> {
    event.to_json()
}

impl Db {
    fn del_event(&self, writer: &mut Writer, event: &Event, uid: &[u8]) -> Result<(), Error> {
        let index_event = event.index();
        let time = index_event.created_at();
        let kind = index_event.kind();
        let pubkey = index_event.pubkey();

        // word
        let bytes = writer.get(&self.t_uid_word, uid)?;
        if let Some(bytes) = bytes {
            let bytes = bytes.to_vec();
            writer.del(&self.t_uid_word, uid, None)?;
            let word = unsafe { rkyv::archived_root::<Vec<Vec<u8>>>(&bytes) };
            for item in word.as_slice() {
                writer.del(&self.t_word, IndexKey::encode_word(item, time), Some(uid))?;
            }
        }

        writer.del(&self.t_data, uid, None)?;
        writer.del(&self.t_index, uid, None)?;
        writer.del(&self.t_id_uid, index_event.id(), None)?;

        writer.del(
            &self.t_id,
            IndexKey::encode_id(index_event.id(), time),
            Some(uid),
        )?;

        writer.del(&self.t_kind, IndexKey::encode_kind(kind, time), Some(uid))?;

        writer.del(
            &self.t_pubkey,
            IndexKey::encode_pubkey(pubkey, time),
            Some(uid),
        )?;
        writer.del(
            &self.t_pubkey_kind,
            IndexKey::encode_pubkey_kind(pubkey, kind, time),
            Some(uid),
        )?;

        if let Some(delegator) = index_event.delegator() {
            writer.del(
                &self.t_pubkey,
                IndexKey::encode_pubkey(delegator, time),
                Some(uid),
            )?;
            writer.del(
                &self.t_pubkey_kind,
                IndexKey::encode_pubkey_kind(delegator, kind, time),
                Some(uid),
            )?;
        }

        writer.del(&self.t_created_at, IndexKey::encode_time(time), Some(uid))?;

        let tagval = concat(uid, kind.to_be_bytes());
        for tag in index_event.tags() {
            writer.del(
                &self.t_tag,
                IndexKey::encode_tag(&tag.0, &tag.1, time),
                Some(&tagval),
            )?;
        }

        // replacement index
        if let Some(k) = encode_replace_key(index_event.kind(), index_event.pubkey(), event.tags())
        {
            writer.del(&self.t_replacement, k, None)?;
        }

        // expiration
        if let Some(t) = index_event.expiration() {
            writer.del(&self.t_expiration, IndexKey::encode_time(*t), Some(uid))?;
        }

        Ok(())
    }

    fn put_event(
        &self,
        writer: &mut Writer,
        event: &Event,
        uid: &Vec<u8>,
        replace_key: &Option<Vec<u8>>,
    ) -> Result<(), Error> {
        let index_event = event.index();

        // put event
        let time = index_event.created_at();
        let json = encode_event(event)?;

        writer.put(&self.t_data, uid, json)?;

        // put index
        let bytes = index_event.to_bytes()?;
        writer.put(&self.t_index, uid, bytes)?;

        // put view
        let kind = index_event.kind();
        let pubkey = index_event.pubkey();

        writer.put(&self.t_id_uid, index_event.id(), uid)?;

        writer.put(&self.t_id, IndexKey::encode_id(index_event.id(), time), uid)?;

        writer.put(&self.t_kind, IndexKey::encode_kind(kind, time), uid)?;

        writer.put(&self.t_pubkey, IndexKey::encode_pubkey(pubkey, time), uid)?;
        writer.put(
            &self.t_pubkey_kind,
            IndexKey::encode_pubkey_kind(pubkey, kind, time),
            uid,
        )?;

        if let Some(delegator) = index_event.delegator() {
            writer.put(
                &self.t_pubkey,
                IndexKey::encode_pubkey(delegator, time),
                uid,
            )?;
            writer.put(
                &self.t_pubkey_kind,
                IndexKey::encode_pubkey_kind(delegator, kind, time),
                uid,
            )?;
        }

        writer.put(&self.t_created_at, IndexKey::encode_time(time), uid)?;

        let tagval = concat(uid, kind.to_be_bytes());
        for tag in index_event.tags() {
            let key = &tag.0;
            let v = &tag.1;
            // tag[0] == 'e'
            if kind == 5 && key[0] == 101 {
                writer.put(&self.t_deletion, concat(index_event.id(), v), uid)?;
            }
            // Provide pubkey kind for filter
            writer.put(&self.t_tag, IndexKey::encode_tag(key, v, time), &tagval)?;
        }

        // replacement index
        if let Some(k) = replace_key {
            // writer.put(&self.t_replacement, k, concat(time.to_be_bytes(), uid))?;
            writer.put(&self.t_replacement, k, uid)?;
        }

        // expiration
        if let Some(t) = index_event.expiration() {
            writer.put(&self.t_expiration, IndexKey::encode_time(*t), uid)?;
        }

        // word
        let words = &event.words;
        if !words.is_empty() {
            let bytes =
                rkyv::to_bytes::<_, 256>(words).map_err(|e| Error::Serialization(e.to_string()))?;
            writer.put(&self.t_uid_word, uid, bytes)?;
            for item in words {
                writer.put(&self.t_word, IndexKey::encode_word(item, time), uid)?;
            }
        }
        Ok(())
    }
}

fn get_event<R: FromEventData, K: AsRef<[u8]>, T: Transaction>(
    reader: &T,
    id_tree: &Tree,
    data_tree: &Tree,
    index_tree: &Tree,
    event_id: K,
) -> Result<Option<(Vec<u8>, R)>, Error> {
    let uid = get_uid(reader, id_tree, event_id)?;
    if let Some(uid) = uid {
        let event = get_event_by_uid(reader, data_tree, index_tree, &uid)?;
        if let Some(event) = event {
            return Ok(Some((uid, event)));
        }
    }
    Ok(None)
}

fn get_event_by_uid<R: FromEventData, K: AsRef<[u8]>, T: Transaction>(
    reader: &T,
    data_tree: &Tree,
    index_tree: &Tree,
    uid: K,
) -> Result<Option<R>, Error> {
    if R::only_id() {
        // get event id from index more faster
        let v = reader.get(index_tree, uid)?;
        let event = decode_event_index(v)?;
        if let Some(v) = event {
            return Ok(Some(
                R::from_data(v.id()).map_err(|e| Error::Message(e.to_string()))?,
            ));
        }
    } else {
        let v = reader.get(data_tree, uid)?;
        if let Some(v) = v {
            return Ok(Some(
                R::from_data(v).map_err(|e| Error::Message(e.to_string()))?,
            ));
        }
    }
    Ok(None)
}

fn decode_event_index(v: Option<&[u8]>) -> Result<Option<&ArchivedEventIndex>, Error> {
    if let Some(v) = v {
        return Ok(Some(EventIndex::from_zeroes(v)?));
    }
    Ok(None)
}

fn get_uid<K: AsRef<[u8]>, T: Transaction>(
    reader: &T,
    id_tree: &Tree,
    event_id: K,
) -> Result<Option<Vec<u8>>, Error> {
    Ok(reader.get(id_tree, event_id)?.map(|v| v.to_vec()))
}

#[derive(Debug, Clone)]
pub enum CheckEventResult {
    Invald(String),
    Duplicate,
    Deleted,
    ReplaceIgnored,
    Ok(usize),
}

impl Db {
    pub fn flush(&self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }

    /// check db version, return [`Error::VersionMismatch`] when db schema changed
    pub fn check_schema(&self) -> Result<()> {
        let mut writer = self.inner.writer()?;
        let old = writer.get(&self.t_meta, "version")?;
        if let Some(old) = old {
            if old != DB_VERSION.as_bytes() {
                return Err(Error::VersionMismatch);
            }
        } else {
            writer.put(&self.t_meta, "version", DB_VERSION)?;
        }
        writer.commit()?;
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = Lmdb::open_with(path, Some(20), Some(100), Some(1_000_000_000_000), 0)?;

        let default_opts = 0;
        // let integer_default_opts = ffi::MDB_INTEGERKEY;
        let integer_default_opts = 0;

        // let index_opts = ffi::MDB_DUPSORT | ffi::MDB_DUPFIXED | ffi::MDB_INTEGERDUP;
        let index_opts = ffi::MDB_DUPSORT | ffi::MDB_DUPFIXED;

        // let integer_index_opts =
        // ffi::MDB_DUPSORT | ffi::MDB_INTEGERKEY | ffi::MDB_DUPFIXED | ffi::MDB_INTEGERDUP;
        // lmdb interger needs check byte order. little-endian
        let integer_index_opts = ffi::MDB_DUPSORT | ffi::MDB_DUPFIXED;

        let t_data = inner.open_tree(Some("t_data"), integer_default_opts)?;
        let t_meta = inner.open_tree(Some("t_meta"), default_opts)?;

        Ok(Self {
            seq: Arc::new(AtomicU64::new(latest_seq(&inner, &t_data)?)),
            t_data,
            t_meta,
            t_index: inner.open_tree(Some("t_index"), integer_default_opts)?,
            t_id_uid: inner.open_tree(Some("t_id_uid"), default_opts)?,
            t_uid_word: inner.open_tree(Some("t_uid_word"), default_opts)?,
            t_deletion: inner.open_tree(Some("t_deletion"), default_opts)?,
            t_replacement: inner.open_tree(Some("t_replacement"), default_opts)?,
            t_id: inner.open_tree(Some("t_id"), default_opts)?,
            t_pubkey: inner.open_tree(Some("t_pubkey"), index_opts)?,
            t_kind: inner.open_tree(Some("t_kind"), index_opts)?,
            t_pubkey_kind: inner.open_tree(Some("t_pubkey_kind"), index_opts)?,
            t_created_at: inner.open_tree(Some("t_created_at"), integer_index_opts)?,
            t_tag: inner.open_tree(Some("t_tag"), ffi::MDB_DUPSORT | ffi::MDB_DUPFIXED)?,
            t_expiration: inner.open_tree(Some("t_expiration"), integer_index_opts)?,
            t_word: inner.open_tree(Some("t_word"), index_opts)?,

            inner,
        })
    }

    pub fn writer(&self) -> Result<Writer> {
        Ok(self.inner.writer()?)
    }

    pub fn reader(&self) -> Result<Reader> {
        Ok(self.inner.reader()?)
    }

    pub fn commit<T: Transaction>(&self, txn: T) -> Result<()> {
        Ok(txn.commit()?)
    }

    pub fn put<E: AsRef<Event>>(&self, writer: &mut Writer, event: E) -> Result<CheckEventResult> {
        let event = event.as_ref();
        let mut count = 0;

        if event.id().len() != 32 || event.pubkey().len() != 32 {
            return Ok(CheckEventResult::Invald(
                "invalid event id or pubkey".to_owned(),
            ));
        }
        // let id: Vec<u8> = pad_start(event.id(), 32);
        let event_id = event.id();
        let pubkey = event.pubkey();

        // Check duplicate event.
        {
            // dup in the db.
            if get_uid(writer, &self.t_id_uid, event_id)?.is_some() {
                return Ok(CheckEventResult::Duplicate);
            }
        }

        // check deleted in db
        if writer
            .get(&self.t_deletion, concat(event_id, pubkey))?
            .is_some()
        {
            return Ok(CheckEventResult::Deleted);
        }

        // [NIP-09](https://nips.be/9)
        // delete event
        if event.kind() == 5 {
            for tag in event.index().tags() {
                if tag.0 == b"e" {
                    // let key = hex::decode(&tag.1).map_err(|e| Error::Hex(e))?;
                    let key = &tag.1;
                    let r = get_event::<Event, _, _>(
                        writer,
                        &self.t_id_uid,
                        &self.t_data,
                        &self.t_index,
                        key,
                    )?;
                    if let Some((uid, e)) = r {
                        // check author or deletion event
                        // check delegator
                        if (e.pubkey() == event.pubkey()
                            || e.index().delegator() == Some(event.pubkey()))
                            && e.kind() != 5
                        {
                            count += 1;
                            self.del_event(writer, &e, &uid)?;
                        }
                    }
                }
            }
        }

        // check replacement event
        let replace_key = encode_replace_key(event.kind(), event.pubkey(), event.tags());

        if let Some(replace_key) = replace_key.as_ref() {
            // lmdb max_key_size 511 bytes
            // we only index tag value length < 255
            if replace_key.len() > MAX_TAG_VALUE_SIZE + 8 + 32 {
                return Ok(CheckEventResult::Invald("invalid replace key".to_owned()));
            }

            // replace in the db
            let v = writer.get(&self.t_replacement, replace_key)?;
            if let Some(v) = v {
                let uid = v.to_vec();
                // let t = &v[0..8];
                // let t = u64_from_bytes(t);
                // if event.created_at() < t {
                //     continue;
                // }
                let e: Option<Event> = get_event_by_uid(writer, &self.t_data, &self.t_index, &uid)?;
                if let Some(e) = e {
                    // If two events have the same timestamp, the event with the lowest id (first in lexical order) SHOULD be retained, and the other discarded.
                    if event.created_at() < e.created_at()
                        || (event.created_at() == e.created_at() && event.id() > e.id())
                    {
                        return Ok(CheckEventResult::ReplaceIgnored);
                    }
                    // del old
                    count += 1;
                    self.del_event(writer, &e, &uid)?;
                }
            }
        }

        count += 1;

        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let seq = u64_to_ver(seq);
        self.put_event(writer, event, &seq, &replace_key)?;
        Ok(CheckEventResult::Ok(count))
    }

    pub fn get<R: FromEventData, K: AsRef<[u8]>, T: Transaction>(
        &self,
        txn: &T,
        event_id: K,
    ) -> Result<Option<R>> {
        let event = get_event(txn, &self.t_id_uid, &self.t_data, &self.t_index, event_id)?;
        Ok(event.map(|e| e.1))
    }

    pub fn del<K: AsRef<[u8]>>(&self, writer: &mut Writer, event_id: K) -> Result<bool> {
        if let Some((uid, event)) = get_event::<Event, _, _>(
            writer,
            &self.t_id_uid,
            &self.t_data,
            &self.t_index,
            event_id,
        )? {
            self.del_event(writer, &event, &uid)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn batch_put<II, N>(&self, events: II) -> Result<usize>
    where
        II: IntoIterator<Item = N>,
        N: AsRef<Event>,
    {
        let mut writer = self.inner.writer()?;
        let mut events = events.into_iter().collect::<Vec<N>>();

        // sort for check dup
        events.sort_by(|a, b| a.as_ref().id().cmp(b.as_ref().id()));
        let mut count = 0;

        for (i, event) in events.iter().enumerate() {
            let event = event.as_ref();
            // dup in the input events
            if i != 0 && event.id() == events[i - 1].as_ref().id() {
                continue;
            }
            if let CheckEventResult::Ok(c) = self.put(&mut writer, event)? {
                count += c;
            }
        }

        writer.commit()?;
        Ok(count)
    }

    pub fn batch_get<R: FromEventData, II, N>(&self, event_ids: II) -> Result<Vec<R>>
    where
        II: IntoIterator<Item = N>,
        N: AsRef<[u8]>,
    {
        let reader = self.reader()?;
        let mut events = vec![];
        for id in event_ids.into_iter() {
            let r = self.get::<R, _, _>(&reader, &id)?;
            if let Some(e) = r {
                events.push(e);
            }
        }
        Ok(events)
    }

    pub fn batch_del<II, N>(&self, event_ids: II) -> Result<()>
    where
        II: IntoIterator<Item = N>,
        N: AsRef<[u8]>,
    {
        let mut writer = self.inner.writer()?;
        for id in event_ids.into_iter() {
            self.del(&mut writer, &id)?;
        }
        writer.commit()?;
        Ok(())
    }

    /// iter events by filter
    pub fn iter<'txn, J: FromEventData, T: Transaction>(
        &self,
        txn: &'txn T,
        filter: &Filter,
    ) -> Result<Iter<'txn, T, J>> {
        if filter.search.as_ref().is_some() {
            let match_index = if !filter.ids.is_empty()
                || !filter.tags.is_empty()
                || !filter.authors.is_empty()
                || !filter.kinds.is_empty()
            {
                MatchIndex::All
            } else {
                MatchIndex::None
            };
            Iter::new_word(self, txn, filter, &self.t_word, match_index)
        } else if !filter.ids.is_empty() {
            let match_index = if !filter.tags.is_empty()
                || !filter.authors.is_empty()
                || !filter.kinds.is_empty()
            {
                MatchIndex::All
            } else {
                MatchIndex::None
            };
            Iter::new_prefix(self, txn, filter, &filter.ids, &self.t_id, match_index)
        } else if !filter.tags.is_empty() {
            let match_index = if !filter.authors.is_empty() {
                MatchIndex::Pubkey
            } else {
                MatchIndex::None
            };
            Iter::new_tag(self, txn, filter, &self.t_tag, match_index)
        } else if !filter.authors.is_empty() && !filter.kinds.is_empty() {
            Iter::new_author_kind(self, txn, filter, &self.t_pubkey_kind, MatchIndex::None)
        } else if !filter.authors.is_empty() {
            Iter::new_prefix(
                self,
                txn,
                filter,
                &filter.authors,
                &self.t_pubkey,
                MatchIndex::None,
            )
        } else if !filter.kinds.is_empty() {
            Iter::new_kind(self, txn, filter, &self.t_kind, MatchIndex::None)
        } else {
            Iter::new_time(self, txn, filter, &self.t_created_at, MatchIndex::None)
        }
    }

    /// iter expired events
    pub fn iter_expiration<'txn, J: FromEventData, T: Transaction>(
        &self,
        txn: &'txn T,
        until: Option<u64>,
    ) -> Result<Iter<'txn, T, J>> {
        let filter = Filter {
            desc: true,
            until,
            ..Default::default()
        };
        Iter::new_time(self, txn, &filter, &self.t_expiration, MatchIndex::None)
    }

    /// iter ephemeral events
    pub fn iter_ephemeral<'txn, J: FromEventData, T: Transaction>(
        &self,
        txn: &'txn T,
        until: Option<u64>,
    ) -> Result<Iter<'txn, T, J>> {
        let filter = Filter {
            desc: false,
            until,
            ..Default::default()
        };
        let mut group = Group::new(filter.desc, false, false);
        let prefix = u16_to_ver(20000);
        let end = u16_to_ver(30000);

        let iter = create_iter(txn, &self.t_kind, &prefix, filter.desc);
        let scanner = Scanner::new(
            iter,
            vec![],
            prefix,
            filter.desc,
            filter.since,
            filter.until,
            Box::new(move |_s, r| {
                let k = r.0;
                let e: &[u8] = end.as_ref();
                Ok(if k < e {
                    MatchResult::Found(IndexKey::from(k, r.1)?)
                } else {
                    MatchResult::Stop
                })
            }),
        );
        group.add(Box::new(scanner))?;
        Iter::new(self, txn, &filter, group, MatchIndex::None)
    }
}

// type IterChecker<I, E> =
//     Box<dyn Fn(&Scanner<I, IndexKey>, &IndexKey) -> Result<CheckResult, Error<E>>>;
// #[allow(unused)]
// enum CheckResult {
//     Continue,
//     Found,
// }

#[derive(Debug)]
enum MatchIndex {
    All,
    Pubkey,
    None,
}

impl MatchIndex {
    fn r#match(&self, filter: &Filter, event: &ArchivedEventIndex) -> bool {
        match &self {
            MatchIndex::Pubkey => {
                Filter::match_author(&filter.authors, event.pubkey(), event.delegator())
            }
            _ => filter.match_archived(event),
        }
    }
}

pub struct Iter<'txn, R, J>
where
    R: Transaction,
{
    reader: &'txn R,
    view_data: Tree,
    view_index: Tree,
    group: Group<'txn, IndexKey, Error>,
    get_data: u64,
    get_index: u64,
    filter: Filter,
    // checker: Option<IterChecker<D::Iter, D::Error>>,
    _r: PhantomData<J>,
    // need get index data for filter
    match_index: MatchIndex,
}

fn create_iter<'a, R: Transaction>(
    reader: &'a R,
    tree: &Tree,
    prefix: &Vec<u8>,
    reverse: bool,
) -> LmdbIter<'a> {
    if reverse {
        let start = upper(prefix.clone())
            .map(Bound::Excluded)
            .unwrap_or(Bound::Unbounded);
        reader.iter_from(tree, start, true)
    } else {
        reader.iter_from(tree, Bound::Included(prefix), false)
    }
}

impl<'txn, R, J> Iter<'txn, R, J>
where
    R: Transaction,
    J: FromEventData,
{
    fn new(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        group: Group<'txn, IndexKey, Error>,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        Ok(Self {
            view_data: kv_db.t_data.clone(),
            view_index: kv_db.t_index.clone(),
            reader,
            group,
            get_data: 0,
            get_index: 0,
            filter: filter.clone(),
            // checker: None,
            _r: PhantomData,
            match_index,
        })
    }

    /// Filter from timestamp index
    fn new_time(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, false, false);
        let prefix = if filter.desc {
            (u64::MAX - 1).to_be_bytes()
        } else {
            0u64.to_be_bytes()
        }
        .to_vec();
        let iter = create_iter(reader, view, &prefix, filter.desc);
        let scanner = Scanner::new(
            iter,
            vec![],
            prefix,
            filter.desc,
            filter.since,
            filter.until,
            Box::new(|_, r| Ok(MatchResult::Found(IndexKey::from(r.0, r.1)?))),
        );
        group.add(Box::new(scanner))?;
        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn new_kind(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, false, false);
        for kind in filter.kinds.iter() {
            let prefix = u16_to_ver(*kind);
            let iter = create_iter(reader, view, &prefix, filter.desc);
            let scanner = Scanner::new(
                iter,
                vec![],
                prefix,
                filter.desc,
                filter.since,
                filter.until,
                Box::new(|s, r| {
                    let k = r.0;
                    Ok(if k.starts_with(&s.prefix) {
                        MatchResult::Found(IndexKey::from(k, r.1)?)
                    } else {
                        MatchResult::Stop
                    })
                }),
            );
            group.add(Box::new(scanner))?;
        }
        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn new_tag(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, true, false);
        let has_kind = !filter.kinds.is_empty();

        for tag in filter.tags.iter() {
            let mut sub = Group::new(filter.desc, false, true);
            for key in tag.1.iter() {
                let kinds = filter.kinds.clone();
                // need add separator to the end, otherwise other tags will intrude
                // ["t", "nostr"]
                // ["t", "nostr1"]
                let prefix = concat_sep(concat_sep(tag.0, key), vec![]);
                let klen = prefix.len() + 8;
                let iter = create_iter(reader, view, &prefix, filter.desc);

                let scanner = Scanner::new(
                    iter,
                    vec![],
                    prefix,
                    filter.desc,
                    filter.since,
                    filter.until,
                    Box::new(move |s, r| {
                        let k = r.0;
                        let v = r.1;
                        Ok(if k.len() == klen && k.starts_with(&s.prefix) {
                            // filter
                            if has_kind && !Filter::match_kind(&kinds, u16_from_bytes(&v[8..10])?) {
                                MatchResult::Continue
                            } else {
                                MatchResult::Found(IndexKey::from(k, v)?)
                            }
                        } else {
                            MatchResult::Stop
                        })
                    }),
                );
                sub.add(Box::new(scanner))?;
            }
            group.add(Box::new(sub))?;
        }
        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn new_author_kind(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, false, false);

        for author in filter.authors.iter() {
            for kind in filter.kinds.iter() {
                let prefix: Vec<u8> = concat(author, u16_to_ver(*kind));
                let iter = create_iter(reader, view, &prefix, filter.desc);
                let scanner = Scanner::new(
                    iter,
                    author.to_vec(),
                    prefix,
                    filter.desc,
                    filter.since,
                    filter.until,
                    Box::new(|s, r| {
                        Ok(if r.0.starts_with(&s.prefix) {
                            MatchResult::Found(IndexKey::from(r.0, r.1)?)
                        } else {
                            MatchResult::Stop
                        })
                    }),
                );
                group.add(Box::new(scanner))?;
            }
        }

        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn new_prefix(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        ids: &[[u8; 32]],
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, false, false);

        for id in ids.iter() {
            let prefix = id.to_vec();
            let iter = create_iter(reader, view, &prefix, filter.desc);
            let scanner = Scanner::new(
                iter,
                prefix.clone(),
                prefix,
                filter.desc,
                filter.since,
                filter.until,
                Box::new(move |s, r| {
                    Ok(if r.0.starts_with(&s.prefix) {
                        MatchResult::Found(IndexKey::from(r.0, r.1)?)
                    } else {
                        MatchResult::Stop
                    })
                }),
            );
            group.add(Box::new(scanner))?;
        }
        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn new_word(
        kv_db: &Db,
        reader: &'txn R,
        filter: &Filter,
        view: &Tree,
        match_index: MatchIndex,
    ) -> Result<Self, Error> {
        let mut group = Group::new(filter.desc, true, true);
        for word in filter.words.iter() {
            let prefix = concat_sep(word, []);
            let klen = prefix.len() + 8;
            let iter = create_iter(reader, view, &prefix, filter.desc);
            let scanner = Scanner::new(
                iter,
                vec![],
                prefix,
                filter.desc,
                filter.since,
                filter.until,
                Box::new(move |s, r| {
                    let k = r.0;
                    Ok(if k.len() == klen && k.starts_with(&s.prefix) {
                        MatchResult::Found(IndexKey::from(k, r.1)?)
                    } else {
                        MatchResult::Stop
                    })
                }),
            );
            group.add(Box::new(scanner))?;
        }
        Self::new(kv_db, reader, filter, group, match_index)
    }

    fn document(&self, key: &IndexKey) -> Result<Option<J>, Error> {
        get_event_by_uid::<J, _, _>(
            self.reader,
            &self.view_data,
            &self.view_index,
            key.uid().to_be_bytes(),
        )
    }

    fn index_data(&self, key: &IndexKey) -> Result<Option<&'txn [u8]>, Error> {
        let v = self.reader.get(&self.view_index, key.uid().to_be_bytes())?;
        Ok(v)
    }

    fn limit(&self, num: u64) -> bool {
        if let Some(limit) = self.filter.limit {
            num >= limit
        } else {
            false
        }
    }

    fn next_inner(&mut self) -> Result<Option<J>, Error> {
        while let Some(item) = self.group.next() {
            let key = item?;
            if matches!(self.match_index, MatchIndex::None) {
                self.get_data += 1;
                if let Some(event) = self.document(&key)? {
                    return Ok(Some(event));
                }
            } else {
                let data = self.index_data(&key)?;
                let event = decode_event_index(data)?;
                self.get_index += 1;
                if let Some(event) = event {
                    if self.match_index.r#match(&self.filter, event) {
                        self.get_data += 1;
                        if let Some(event) = self.document(&key)? {
                            return Ok(Some(event));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

impl<'txn, R, J> Iter<'txn, R, J>
where
    R: Transaction,
    J: FromEventData,
{
    /// Limit the total scan time and report [`Error::ScanTimeout`] if it is exceeded
    pub fn scan_time(&mut self, timeout: Duration, check_step: u64) {
        let start = Instant::now();
        let mut last = check_step;
        self.group.watcher(Box::new(move |count| {
            if count > last {
                // check
                if start.elapsed() > timeout {
                    return Err(Error::ScanTimeout);
                }
                last = count + check_step;
            }
            Ok(())
        }));
    }

    /// The stats after scan
    pub fn stats(&self) -> Stats {
        Stats {
            scan_index: self.group.scan_times,
            get_data: self.get_data,
            get_index: self.get_index,
        }
    }

    /// only count iter size
    pub fn size(mut self) -> Result<(u64, Stats)> {
        let mut len = 0;
        while let Some(item) = self.group.next() {
            let key = item?;
            if matches!(self.match_index, MatchIndex::None) {
                len += 1;
                if self.limit(len) {
                    break;
                }
            } else {
                let data = self.index_data(&key)?;
                let event = decode_event_index(data)?;
                self.get_index += 1;
                if let Some(event) = event {
                    if self.match_index.r#match(&self.filter, event) {
                        len += 1;
                        if self.limit(len) {
                            break;
                        }
                    }
                }
            }
        }
        Ok((
            len,
            Stats {
                get_data: 0,
                get_index: self.get_index,
                scan_index: self.group.scan_times,
            },
        ))
    }
}

impl<'txn, R, J> Iterator for Iter<'txn, R, J>
where
    R: Transaction,
    J: FromEventData,
{
    type Item = Result<J, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.limit(self.get_data) {
            None
        } else {
            self.next_inner().transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::upper;

    #[test]
    pub fn test_upper_fn() {
        assert_eq!(upper(vec![1, 2, 3, 4, 5]), Some(vec![1, 2, 3, 4, 6]));
        assert_eq!(upper(vec![1, 2, 3, 4, 255]), Some(vec![1, 2, 3, 5]));
        assert_eq!(upper(vec![1, 2, 3, 255, 255]), Some(vec![1, 2, 4]));
        assert_eq!(upper(vec![1, 2, 255, 255, 255]), Some(vec![1, 3]));
        assert_eq!(upper(vec![1, 255, 255, 255, 255]), Some(vec![2]));
        assert_eq!(upper(vec![255, 255, 255, 255, 255]), None);
        assert_eq!(upper(vec![1, 2, 3, 255, 5]), Some(vec![1, 2, 3, 255, 6]));
        assert_eq!(upper(vec![255, 2, 3, 4, 5]), Some(vec![255, 2, 3, 4, 6]));
    }
}
```

---

### error.rs

**Size:** 1214 bytes | **Modified:** 2025-11-22 21:18:16

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Kv(#[from] nostr_kv::Error),
    #[error(transparent)]
    ConvertU64(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("hex: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("deserialization: {0}")]
    Deserialization(String),
    #[error("serialization: {0}")]
    Serialization(String),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("invalid length")]
    InvalidLength,
    #[error("message: {0}")]
    Message(String),
    #[error("Scan timeout")]
    ScanTimeout,
    #[error("The database schema has been modified. Please run export first, move the old database file, then import and start the program.
      Find the rnostr command at https://github.com/rnostr/rnostr#commands
      rnostr export data/events > events.json
      mv data/events data/old_events
      rnostr import data/events events.json
    ")]
    VersionMismatch,
}
```

---

### event.rs

**Size:** 25668 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use crate::error::Error;
use rkyv::{
    vec::ArchivedVec, AlignedVec, Archive, Archived, Deserialize as RkyvDeserialize,
    Serialize as RkyvSerialize,
};
use secp256k1::{schnorr::Signature, Keypair, Message, XOnlyPublicKey, SECP256K1};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fmt::Display,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

type Tags = Vec<(Vec<u8>, Vec<u8>)>;
type BuildTags = (Tags, Option<u64>, Option<[u8; 32]>);
#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    Default,
    Archive,
    RkyvDeserialize,
    RkyvSerialize,
)]
pub struct EventIndex {
    #[serde(with = "hex::serde")]
    id: [u8; 32],

    #[serde(with = "hex::serde")]
    pubkey: [u8; 32],

    created_at: u64,

    kind: u16,

    #[serde(skip)]
    tags: Tags,

    #[serde(skip)]
    expiration: Option<u64>,

    /// [NIP-26](https://nips.be/26)
    #[serde(skip)]
    delegator: Option<[u8; 32]>,
}

impl EventIndex {
    pub fn from_zeroes(bytes: &[u8]) -> Result<&ArchivedEventIndex, Error> {
        let archived = unsafe { rkyv::archived_root::<Self>(bytes) };
        Ok(archived)
    }

    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self, Error> {
        let bytes = bytes.as_ref();
        let archived = unsafe { rkyv::archived_root::<Self>(bytes) };
        let deserialized: Self = archived
            .deserialize(&mut rkyv::Infallible)
            .map_err(|e| Error::Deserialization(e.to_string()))?;
        Ok(deserialized)
    }

    pub fn to_bytes(&self) -> Result<AlignedVec, Error> {
        let vec =
            rkyv::to_bytes::<_, 256>(self).map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(vec)
    }

    pub fn new(
        id: [u8; 32],
        pubkey: [u8; 32],
        created_at: u64,
        kind: u16,
        tags: &Vec<Vec<String>>,
    ) -> Result<Self, Error> {
        let (tags, expiration, delegator) = Self::build_index_tags(tags)?;
        Ok(Self {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            expiration,
            delegator,
        })
    }

    pub fn build_index_tags(tags: &Vec<Vec<String>>) -> Result<BuildTags, Error> {
        let mut t = vec![];
        let mut expiration = None;
        let mut delegator = None;

        for tag in tags {
            if tag.len() > 1 {
                if tag[0] == "expiration" {
                    expiration = Some(
                        u64::from_str(&tag[1])
                            .map_err(|_| Error::Invalid("invalid expiration".to_string()))?,
                    );
                } else if tag[0] == "delegation" {
                    let mut h = [0u8; 32];
                    hex::decode_to_slice(&tag[1], &mut h)?;
                    delegator = Some(h);
                }

                let key = tag[0].as_bytes().to_vec();
                // only index key length 1
                // 0 will break the index separator, ignore
                if key.len() == 1 && key[0] != 0 {
                    let v;
                    // fixed length 32 e and p
                    if tag[0] == "e" || tag[0] == "p" {
                        let h = hex::decode(&tag[1])?;
                        if h.len() != 32 {
                            return Err(Error::Invalid("invalid e or p tag value".to_string()));
                        }
                        v = h;
                    } else {
                        v = tag[1].as_bytes().to_vec();
                        // 0 will break the index separator, ignore
                        // lmdb max_key_size 511 bytes
                        // we only index tag value length < 255
                        if v.contains(&0) || v.len() > 255 {
                            continue;
                        }
                    };
                    t.push((key, v));
                }
            }
        }
        Ok((t, expiration, delegator))
    }

    pub fn id(&self) -> &[u8; 32] {
        &self.id
    }

    pub fn pubkey(&self) -> &[u8; 32] {
        &self.pubkey
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn kind(&self) -> u16 {
        self.kind
    }

    pub fn tags(&self) -> &Vec<(Vec<u8>, Vec<u8>)> {
        &self.tags
    }

    pub fn expiration(&self) -> Option<&u64> {
        self.expiration.as_ref()
    }

    pub fn delegator(&self) -> Option<&[u8; 32]> {
        self.delegator.as_ref()
    }

    pub fn is_ephemeral(&self) -> bool {
        let kind = self.kind;
        (20_000..30_000).contains(&kind)
    }

    pub fn is_expired(&self, now: u64) -> bool {
        if let Some(exp) = self.expiration {
            exp < now
        } else {
            false
        }
    }
}

impl ArchivedEventIndex {
    pub fn id(&self) -> &Archived<[u8; 32]> {
        &self.id
    }
    pub fn pubkey(&self) -> &Archived<[u8; 32]> {
        &self.pubkey
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn kind(&self) -> u16 {
        self.kind
    }

    pub fn tags(&self) -> &ArchivedVec<(ArchivedVec<u8>, ArchivedVec<u8>)> {
        &self.tags
    }

    pub fn expiration(&self) -> Option<&u64> {
        self.expiration.as_ref()
    }

    pub fn delegator(&self) -> Option<&Archived<[u8; 32]>> {
        self.delegator.as_ref()
    }

    pub fn is_ephemeral(&self) -> bool {
        let kind = self.kind;
        (20_000..30_000).contains(&kind)
    }

    pub fn is_expired(&self, now: u64) -> bool {
        if let Some(exp) = self.expiration.as_ref() {
            exp < &now
        } else {
            false
        }
    }
}
// the shadow event for deserialize
#[derive(Deserialize)]
struct _Event {
    #[serde(with = "hex::serde")]
    id: [u8; 32],
    #[serde(with = "hex::serde")]
    pubkey: [u8; 32],
    created_at: u64,
    kind: u16,
    #[serde(default)]
    tags: Vec<Vec<String>>,
    #[serde(default)]
    content: String,
    #[serde(with = "hex::serde")]
    sig: [u8; 64],
    // #[serde(flatten)]
    // index: IndexEvent,
}

/// The default event document.
// TODO: validate index tag value length 255
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "_Event")]
pub struct Event {
    #[serde(default)]
    tags: Vec<Vec<String>>,

    #[serde(default)]
    content: String,

    #[serde(with = "hex::serde")]
    sig: [u8; 64],

    #[serde(flatten)]
    index: EventIndex,

    #[serde(skip)]
    pub words: Vec<Vec<u8>>,
}

impl TryFrom<_Event> for Event {
    type Error = Error;

    fn try_from(value: _Event) -> Result<Self, Self::Error> {
        let event = Event {
            content: value.content,
            sig: value.sig,
            index: EventIndex::new(
                value.id,
                value.pubkey,
                value.created_at,
                value.kind,
                &value.tags,
            )?,
            tags: value.tags,
            words: Default::default(),
        };
        Ok(event)
    }
}

impl Event {
    pub fn new(
        id: [u8; 32],
        pubkey: [u8; 32],
        created_at: u64,
        kind: u16,
        tags: Vec<Vec<String>>,
        content: String,
        sig: [u8; 64],
    ) -> Result<Self, Error> {
        let index = EventIndex::new(id, pubkey, created_at, kind, &tags)?;
        let event = Self {
            tags,
            content,
            sig,
            index,
            words: Default::default(),
        };
        Ok(event)
    }

    pub fn create(
        key_pair: &Keypair,
        created_at: u64,
        kind: u16,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> Result<Self, Error> {
        let pubkey = XOnlyPublicKey::from_keypair(key_pair).0.serialize();
        let id = hash(&pubkey, created_at, kind, &tags, &content);
        let sig = *SECP256K1
            .sign_schnorr(&Message::from_digest_slice(&id)?, key_pair)
            .as_ref();
        Self::new(id, pubkey, created_at, kind, tags, content, sig)
    }
}

impl AsRef<Event> for Event {
    fn as_ref(&self) -> &Event {
        self
    }
}

impl FromStr for Event {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = serde_json::to_string(&self).unwrap();
        f.write_str(&str)?;
        Ok(())
    }
}

impl TryInto<String> for Event {
    type Error = Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}

pub trait FromEventData: Sized {
    type Err: std::error::Error;
    /// only pass the event id to from_data
    fn only_id() -> bool {
        false
    }
    fn from_data<S: AsRef<[u8]>>(data: S) -> Result<Self, Self::Err>;
}

/// Get the event id
impl FromEventData for Vec<u8> {
    type Err = Error;
    fn only_id() -> bool {
        true
    }
    fn from_data<S: AsRef<[u8]>>(data: S) -> Result<Self, Self::Err> {
        Ok(data.as_ref().to_vec())
    }
}

/// Get the json string
impl FromEventData for String {
    type Err = Error;
    fn from_data<S: AsRef<[u8]>>(json: S) -> Result<Self, Self::Err> {
        let (t, bytes) = parse_data_type(json.as_ref());
        if t == 1 {
            #[cfg(feature = "zstd")]
            {
                let bytes = zstd::decode_all(bytes)?;
                Ok(unsafe { String::from_utf8_unchecked(bytes) })
            }
            #[cfg(not(feature = "zstd"))]
            {
                Err(Error::Invalid("Need zstd feature".to_owned()))
            }
        } else {
            Ok(unsafe { String::from_utf8_unchecked(bytes.to_vec()) })
        }
    }
}

fn parse_data_type(json: &[u8]) -> (u8, &[u8]) {
    if !json.is_empty() {
        let last = json.len() - 1;
        let t = json[last];
        if t == 0 || t == 1 {
            return (t, &json[0..last]);
        }
    }
    (0, json)
}

/// Parse the json string to event object
impl FromEventData for Event {
    type Err = Error;
    /// decode the json data to event object
    fn from_data<S: AsRef<[u8]>>(json: S) -> Result<Self, Self::Err> {
        let (t, bytes) = parse_data_type(json.as_ref());
        if t == 1 {
            #[cfg(feature = "zstd")]
            {
                let bytes = zstd::decode_all(bytes)?;
                Ok(serde_json::from_slice(&bytes)?)
            }
            #[cfg(not(feature = "zstd"))]
            {
                Err(Error::Invalid("Need zstd feature".to_owned()))
            }
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }
}

#[cfg(feature = "search")]
impl Event {
    /// build keywords for search ability
    pub fn build_note_words(&mut self) {
        if self.kind() == 1 {
            let mut words = crate::segment(&self.content);
            self.words.append(&mut words);
        }
    }
}

impl Event {
    /// to json string
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    pub fn index(&self) -> &EventIndex {
        &self.index
    }

    pub fn id(&self) -> &[u8; 32] {
        &self.index.id
    }

    pub fn id_str(&self) -> String {
        hex::encode(self.index.id)
    }

    pub fn pubkey(&self) -> &[u8; 32] {
        &self.index.pubkey
    }

    pub fn pubkey_str(&self) -> String {
        hex::encode(self.index.pubkey)
    }

    pub fn created_at(&self) -> u64 {
        self.index.created_at
    }

    pub fn kind(&self) -> u16 {
        self.index.kind
    }

    pub fn tags(&self) -> &Vec<Vec<String>> {
        &self.tags
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn sig(&self) -> &[u8; 64] {
        &self.sig
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hash(
    pubkey: &[u8],
    created_at: u64,
    kind: u16,
    tags: &Vec<Vec<String>>,
    content: &String,
) -> [u8; 32] {
    let json: Value = json!([0, hex::encode(pubkey), created_at, kind, tags, content]);
    let mut hasher = Sha256::new();
    hasher.update(json.to_string());
    hasher.finalize().into()
}

impl Event {
    pub fn hash(&self) -> [u8; 32] {
        hash(
            self.pubkey(),
            self.created_at(),
            self.kind(),
            self.tags(),
            self.content(),
        )
    }

    pub fn verify_id(&self) -> Result<(), Error> {
        if &self.hash() == self.id() {
            Ok(())
        } else {
            Err(Error::Invalid("bad event id".to_owned()))
        }
    }

    pub fn verify_sign(&self) -> Result<(), Error> {
        if verify_sign(&self.sig, self.pubkey(), self.id()).is_ok() {
            Ok(())
        } else {
            Err(Error::Invalid("signature is wrong".to_owned()))
        }
    }

    /// check event created time newer than (now - older), older than (now + newer)
    /// ignore when 0
    pub fn verify_time(&self, now: u64, older: u64, newer: u64) -> Result<(), Error> {
        let time = self.created_at();
        if 0 != older && time < now - older {
            return Err(Error::Invalid(format!(
                "event creation date must be newer than {}",
                now - older
            )));
        }

        if 0 != newer && time > now + newer {
            return Err(Error::Invalid(format!(
                "event creation date must be older than {}",
                now + newer
            )));
        }
        Ok(())
    }

    pub fn verify_delegation(&self) -> Result<(), Error> {
        if self.index.delegator.is_some() {
            for tag in self.tags() {
                if tag.len() == 4 && tag[0] == "delegation" {
                    return verify_delegation(self, &tag[1], &tag[2], &tag[3]);
                }
            }
            Err(Error::Invalid("error delegation arguments".to_owned()))
        } else {
            Ok(())
        }
    }

    pub fn validate(&self, now: u64, older: u64, newer: u64) -> Result<(), Error> {
        if self.index.is_expired(now) {
            return Err(Error::Invalid("event is expired".to_owned()));
        }
        self.verify_time(now, older, newer)?;
        self.verify_id()?;
        self.verify_sign()?;
        self.verify_delegation()?;
        Ok(())
    }
}

fn verify_delegation(
    event: &Event,
    delegator: &String,
    conditions: &String,
    sig: &String,
) -> Result<(), Error> {
    let msg = format!(
        "nostr:delegation:{}:{}",
        hex::encode(event.pubkey()),
        conditions
    );
    let mut hasher = Sha256::new();
    hasher.update(msg);
    let token = hasher.finalize().to_vec();
    verify_sign(&hex::decode(sig)?, &hex::decode(delegator)?, &token)?;
    let time = event.created_at();
    // check conditions
    for cond in conditions.split('&') {
        if let Some(kind) = cond.strip_prefix("kind=") {
            let n = u16::from_str(kind)?;
            if n != event.kind() {
                return Err(Error::Invalid(format!(
                    "event kind must be {}",
                    event.kind()
                )));
            }
        }
        if let Some(t) = cond.strip_prefix("created_at<") {
            let n = u64::from_str(t)?;
            if time >= n {
                return Err(Error::Invalid(format!(
                    "event created_at must older than {}",
                    n
                )));
            }
        }
        if let Some(t) = cond.strip_prefix("created_at>") {
            let n = u64::from_str(t)?;
            if time <= n {
                return Err(Error::Invalid(format!(
                    "event created_at must newer than {}",
                    n
                )));
            }
        }
    }

    Ok(())
}

fn verify_sign(sig: &[u8], pk: &[u8], msg: &[u8]) -> Result<(), Error> {
    let sig = Signature::from_slice(sig)?;
    let pk = XOnlyPublicKey::from_slice(pk)?;
    let msg = Message::from_digest_slice(msg)?;
    Ok(SECP256K1.verify_schnorr(&sig, &msg, &pk)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use secp256k1::rand::thread_rng;
    use serde_json::Value;
    use std::str::FromStr;

    #[test]
    fn index_event() -> Result<()> {
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"], ["t", ""], ["expiration", "1"], ["delegation", "8e0d3d3eb2881ec137a11debe736a9086715a8c8beeeda615780064d68bc25dd"]]
          }
        "#;
        let event: Event = Event::from_str(note)?;
        assert_eq!(event.index().tags().len(), 2);
        let e2 = EventIndex::from_bytes(&event.index().to_bytes()?)?;
        assert_eq!(&e2, event.index());
        assert!(&e2.expiration().is_some());
        assert!(&e2.delegator().is_some());

        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": []
          }
        "#;
        let event: Event = Event::from_str(note)?;
        assert_eq!(event.index().tags().len(), 0);
        let e2 = EventIndex::from_bytes(&event.index().to_bytes()?)?;
        assert_eq!(&e2, event.index());
        Ok(())
    }

    #[test]
    fn string() -> Result<()> {
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"]]
          }
        "#;
        let event: Event = Event::from_str(note)?;
        assert_eq!(
            hex::encode(event.index().id()),
            "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d"
        );
        assert_eq!(
            hex::encode(event.index().id()),
            "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d"
        );
        assert_eq!(event.index().id().len(), 32);
        let json: String = event.try_into()?;
        let val: Value = serde_json::from_str(&json)?;
        assert_eq!(
            val["id"],
            Value::String(
                "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d".to_string()
            )
        );
        Ok(())
    }
    #[test]
    fn deserialize() -> Result<()> {
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"]]
          }
        "#;
        let event: Event = serde_json::from_str(note)?;
        assert_eq!(
            hex::encode(event.index().id()),
            "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d"
        );
        assert_eq!(
            hex::encode(event.index().id()),
            "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d"
        );
        assert_eq!(event.index().id().len(), 32);
        assert_eq!(&event.tags, &vec![vec!["t", "nostr"]]);
        assert_eq!(event.index().tags.len(), 1);

        // null tag
        let note = r#"
        {"content":"","created_at":1681838474,"id":"bf2b783de44b814778d02ca9e4e87aacd0bc7a629bad29b5db62a1c151580ed1","kind":1,"pubkey":"d477a41316e6d28c469181690237705024eb313b43ed3e1f059dc2ff49a6dd2f","sig":"96fa5e33aefd4b18f2d5ab5dc199e731fd6c33162ef3eeee945959b98901e80d1b8fb62856f4f0baed166f4aab2d4401aa8ce9e48071dbe220d2b8e9773755de","tags":[["e","fad5161223be749e364f0eac0fc8cf1566659a32c75d9ce388be42c36ac33e44",null,"root"]]}
        "#;
        let event = Event::from_str(note);
        assert!(event.is_err());

        // invalid kind
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 65536,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"]]
          }
        "#;
        let event = Event::from_str(note);
        assert!(event.is_err());

        Ok(())
    }

    #[test]
    fn default() -> Result<()> {
        let note = r#"
        {
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f"
          }
        "#;
        let event: Event = serde_json::from_str(note)?;
        assert_eq!(&event.content, "");
        assert_eq!(&event.tags, &Vec::<Vec<String>>::new());
        Ok(())
    }

    #[test]
    fn verify() -> Result<()> {
        let note = r#"
        {"content":"bgQih8o+R83t00qvueD7twglJRvvabI+nDu+bTvRsAs=?iv=92TlqnpEeiUMzDtUxsZeUA==","created_at":1682257003,"id":"dba1951f0959dfea6e3123ad916d191a07b35392c4b541d4b4814e77113de14a","kind":4,"pubkey":"3f770d65d3a764a9c5cb503ae123e62ec7598ad035d836e2a810f3877a745b24","sig":"15dcc89bca7d037d6a5282c1e63ea40ca4f76d81821ca1260898a324c99516a0cb577617cf18a3febe6303ed32e7a1a08382eecde5a7183195ca8f186a0cb037","tags":[["p","6efb74e66b7ed7fb9fb7b8b8f12e1fbbabe7f45823a33a14ac60cc9241285536"]]}
        "#;
        let event: Event = serde_json::from_str(note)?;
        assert!(event.verify_sign().is_ok());
        assert!(event.verify_id().is_ok());
        assert!(!event.index().is_expired(now()));
        assert!(!event.index().is_ephemeral());

        let note = r#"
        {"content":"{\"display_name\": \"maglevclient\", \"uptime\": 103180, \"maglev\": \"1a98030114cf\"}","created_at":1682258083,"id":"153a480d7bb9d7564147241b330a8667b19c3f9178b8179e64bf57f200654cb0","kind":0,"pubkey":"fb7324a1b807b48756be8df06bd9ccf11741a9678b120e91e044b5137734dcb2","sig":"08c0ffa072fd49f405df467ccab25152a54073fc0639ea0952e1eabff7962e008c54cb8f4d2d55dc4398703df4a5654d2ae3e93f68a801bcbabcdb8050a918ef","tags":[["t","TESTmaglev"],["expiration","1682258683"]]}
          "#;
        let event: Event = serde_json::from_str(note)?;
        assert!(event.verify_sign().is_ok());
        assert!(event.verify_id().is_ok());
        assert!(event.index().is_expired(now()));

        let event = Event::new([0; 32], [0; 32], 10, 1, vec![], "".to_string(), [0; 64])?;
        assert!(event.verify_time(10, 1, 1).is_ok());
        assert!(event.verify_time(20, 1, 1).is_err());
        assert!(event.verify_time(5, 1, 1).is_err());

        let note = r#"
        {
            "id": "e93c6095c3db1c31d15ac771f8fc5fb672f6e52cd25505099f62cd055523224f",
            "pubkey": "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
            "created_at": 1677426298,
            "kind": 1,
            "tags": [
              [
                "delegation",
                "8e0d3d3eb2881ec137a11debe736a9086715a8c8beeeda615780064d68bc25dd",
                "kind=1&created_at>1674834236&created_at<1677426236",
                "6f44d7fe4f1c09f3954640fb58bd12bae8bb8ff4120853c4693106c82e920e2b898f1f9ba9bd65449a987c39c0423426ab7b53910c0c6abfb41b30bc16e5f524"
              ]
            ],
            "content": "Hello, world!",
            "sig": "633db60e2e7082c13a47a6b19d663d45b2a2ebdeaf0b4c35ef83be2738030c54fc7fd56d139652937cdca875ee61b51904a1d0d0588a6acd6168d7be2909d693"
          }
        "#;
        let event: Event = serde_json::from_str(note)?;
        assert!(event.verify_delegation().is_err());
        assert!(event
            .verify_delegation()
            .unwrap_err()
            .to_string()
            .contains("older"));

        Ok(())
    }

    #[test]
    fn create() -> Result<()> {
        let mut rng = thread_rng();
        let key_pair = Keypair::new_global(&mut rng);
        let event = Event::create(&key_pair, 0, 1, vec![], "".to_owned())?;
        assert!(event.verify_sign().is_ok());
        assert!(event.verify_id().is_ok());
        Ok(())
    }
}
```

---

### filter.rs

**Size:** 18014 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use crate::{error::Error, ArchivedEventIndex, EventIndex};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ord;
use std::{collections::HashMap, ops::Deref, str::FromStr};

/// The sort list contains unduplicated and sorted items
#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct SortList<T>(Vec<T>);

impl<T: Ord> From<Vec<T>> for SortList<T> {
    fn from(mut value: Vec<T>) -> Self {
        value.sort();
        value.dedup();
        Self(value)
    }
}

impl<T: Ord> SortList<T> {
    pub fn contains(&self, item: &T) -> bool {
        self.binary_search(item).is_ok()
    }
}

impl<T: Ord + AsRef<[u8]>> SortList<T> {
    pub fn contains2<I: AsRef<[u8]>>(&self, item: I) -> bool {
        self.binary_search_by(|p| p.as_ref().cmp(item.as_ref()))
            .is_ok()
    }
}

impl<T> Deref for SortList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Events filter
///
/// [NIP-01](https://nips.be/1)

// TODO: hashset uniq, (default limit), limit length, limit item length, empty string, invald hex prefix, validate length
#[derive(PartialEq, Eq, Debug, Clone, Default, Deserialize)]
#[serde(try_from = "_Filter")]
pub struct Filter {
    /// a list of event ids
    pub ids: SortList<[u8; 32]>,

    /// a list of pubkeys, the pubkey of an event must be one of these
    pub authors: SortList<[u8; 32]>,

    /// a list of a kind numbers
    pub kinds: SortList<u16>,

    pub since: Option<u64>,
    pub until: Option<u64>,
    pub limit: Option<u64>,

    /// Keyword search  [NIP-50](https://nips.be/50) , [keywords renamed to search](https://github.com/nostr-protocol/nips/commit/6708a73bbcd141094c75f739c8b31446620b30e1)
    pub search: Option<String>,

    /// tags starts with "#", key tag length 1
    ///
    pub tags: HashMap<Vec<u8>, SortList<Vec<u8>>>,

    /// Query by time descending order
    pub desc: bool,

    #[serde(skip)]
    pub words: Vec<Vec<u8>>,
}

impl FromStr for Filter {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct _Filter {
    pub ids: Vec<_HexString>,
    pub authors: Vec<_HexString>,
    pub kinds: Vec<u16>,
    pub since: Option<u64>,
    pub until: Option<u64>,
    pub limit: Option<u64>,
    pub keywords: Vec<String>,
    pub search: Option<String>,
    #[serde(flatten)]
    pub tags: HashMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct _HexString {
    #[serde(with = "hex::serde")]
    hex: [u8; 32],
}

impl TryFrom<_Filter> for Filter {
    type Error = Error;
    fn try_from(filter: _Filter) -> Result<Self, Self::Error> {
        // deserialize search option, convert keywords array to string
        let mut search = filter.search;
        if search.is_none() && !filter.keywords.is_empty() {
            search = Some(filter.keywords.join(" "));
        }

        // only use valid tag, has prefix "#", string item, not empty
        let mut tags = HashMap::new();
        for item in filter.tags {
            let key = item.0;
            if let Some(key) = key.strip_prefix('#') {
                let key = key.as_bytes();
                // only index for key len 1
                if key.len() == 1 {
                    let val = Vec::<String>::deserialize(&item.1)?;
                    let mut list = vec![];
                    for s in val {
                        if key == b"e" || key == b"p" {
                            let h = hex::decode(&s)?;
                            if h.len() != 32 {
                                // ignore
                                return Err(Error::Invalid("invalid e or p tag value".to_string()));
                            } else {
                                list.push(h);
                            }
                        } else {
                            list.push(s.into_bytes());
                            // if s.len() < 255 {
                            // } else {
                            //     return Err(Error::Invald("invalid value length".to_string()));
                            // }
                        }
                    }
                    if !list.is_empty() {
                        tags.insert(key.to_vec(), list.into());
                    }
                }
            }
        }

        let f = Filter {
            ids: filter
                .ids
                .into_iter()
                .map(|s| s.hex)
                .collect::<Vec<_>>()
                .into(),
            authors: filter
                .authors
                .into_iter()
                .map(|s| s.hex)
                .collect::<Vec<_>>()
                .into(),
            kinds: filter.kinds.into(),
            since: filter.since,
            until: filter.until,
            limit: filter.limit,
            search,
            tags,
            desc: filter.limit.is_some(),
            words: vec![],
        };

        Ok(f)
    }
}

impl Filter {
    #[cfg(feature = "search")]
    /// build keywords for search ability
    pub fn build_words(&mut self) {
        if let Some(search) = &self.search {
            let words = crate::segment(search);
            if !words.is_empty() {
                self.words = words;
            }
        }
    }

    pub fn default_limit(&mut self, limit: u64) {
        if self.limit.is_none() {
            self.limit = Some(limit);
        }
    }

    pub fn set_tags(&mut self, tags: HashMap<String, Vec<String>>) {
        let mut t = HashMap::new();
        for item in tags {
            let key = item.0.into_bytes();
            // only index for key len 1
            if key.len() == 1 {
                let val = item
                    .1
                    .into_iter()
                    .map(|s| s.into_bytes())
                    // only index tag value length < 255
                    .filter(|s| s.len() < 255)
                    .collect::<Vec<_>>();
                if !key.is_empty() && !val.is_empty() {
                    t.insert(key, val.into());
                }
            }
        }
        self.tags = t;
    }

    pub fn match_id(ids: &SortList<[u8; 32]>, id: &[u8; 32]) -> bool {
        ids.is_empty() || ids.contains(id)
    }

    pub fn match_author(
        authors: &SortList<[u8; 32]>,
        pubkey: &[u8; 32],
        delegator: Option<&[u8; 32]>,
    ) -> bool {
        authors.is_empty()
            || Self::match_id(authors, pubkey)
            || delegator
                .map(|d| Self::match_id(authors, d))
                .unwrap_or_default()
    }

    pub fn match_kind(kinds: &SortList<u16>, kind: u16) -> bool {
        kinds.is_empty() || kinds.contains(&kind)
    }

    pub fn match_tag<V: AsRef<[u8]>, I: AsRef<[(V, V)]>>(
        tags: &HashMap<Vec<u8>, SortList<Vec<u8>>>,
        event_tags: I,
    ) -> bool {
        // empty tags
        if tags.is_empty() {
            return true;
        }

        // event has not tag
        if event_tags.as_ref().is_empty() {
            return false;
        }

        // all tag must match
        for tag in tags.iter() {
            if !Self::tag_contains(&event_tags, tag.0, tag.1) {
                return false;
            }
        }
        true
    }

    fn tag_contains<V: AsRef<[u8]>, I: AsRef<[(V, V)]>>(
        tags: I,
        name: &[u8],
        list: &SortList<Vec<u8>>,
    ) -> bool {
        let tags = tags.as_ref();
        if tags.is_empty() {
            return false;
        }
        for tag in tags {
            if tag.0.as_ref() == name && list.contains2(tag.1.as_ref()) {
                return true;
            }
        }
        false
    }

    pub fn r#match(&self, event: &EventIndex) -> bool {
        self.match_except_tag(event) && Self::match_tag(&self.tags, event.tags())
    }

    pub fn match_except_tag(&self, event: &EventIndex) -> bool {
        Self::match_id(&self.ids, event.id())
            && self.since.map_or(true, |t| event.created_at() >= t)
            && self.until.map_or(true, |t| event.created_at() <= t)
            && Self::match_kind(&self.kinds, event.kind())
            && Self::match_author(&self.authors, event.pubkey(), event.delegator())
    }

    pub fn match_archived(&self, event: &ArchivedEventIndex) -> bool {
        self.match_archived_except_tag(event) && Self::match_tag(&self.tags, event.tags())
    }

    pub fn match_archived_except_tag(&self, event: &ArchivedEventIndex) -> bool {
        Self::match_id(&self.ids, event.id())
            && self.since.map_or(true, |t| event.created_at() >= t)
            && self.until.map_or(true, |t| event.created_at() <= t)
            && Self::match_kind(&self.kinds, event.kind())
            && Self::match_author(&self.authors, event.pubkey(), event.delegator())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use super::Filter;
    use crate::{filter::SortList, ArchivedEventIndex, Event, EventIndex};
    use anyhow::Result;

    #[test]
    fn deser_filter() -> Result<()> {
        // empty
        let note = "{}";
        let filter: Filter = serde_json::from_str(note)?;
        assert!(filter.tags.is_empty());
        assert!(filter.ids.is_empty());

        // valid
        let note = r###"
        {
            "ids": ["abababababababababababababababababababababababababababababababab", "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd", "1212121212121212121212121212121212121212121212121212121212121212"],
            "authors": ["abababababababababababababababababababababababababababababababab", "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd", "1212121212121212121212121212121212121212121212121212121212121212"],
            "kinds": [2, 1],
            "until": 5,
            "since": 3,
            "limit": 6,
            "#d": ["ab", "cd", "12"],
            "#f": ["ab", "cd", "12", "ab"],
            "#b": [],
            "search": "abc",
            "invalid": ["ab", "cd", "12"],
            "_invalid": 123
          }
        "###;
        let mut filter: Filter = serde_json::from_str(note)?;
        let li = SortList::from(vec![[0x12; 32], [0xab; 32], [0xcd; 32]]);
        let tags: SortList<Vec<u8>> = ["ab", "cd", "12"]
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .into();
        assert_eq!(&filter.ids, &li);
        assert_eq!(&filter.authors, &li);
        assert_eq!(&filter.kinds, &SortList::from(vec![1, 2]));
        assert_eq!(filter.until, Some(5));
        assert_eq!(filter.since, Some(3));
        assert_eq!(filter.limit, Some(6));
        assert_eq!(filter.search, Some("abc".to_string()));

        // tag
        assert_eq!(
            &filter.tags.get(&"d".to_string().into_bytes()),
            &Some(&tags)
        );
        // dup
        assert_eq!(
            &filter.tags.get(&"f".to_string().into_bytes()),
            &Some(&tags)
        );
        assert!(filter
            .tags
            .get(&"invalid".to_string().into_bytes())
            .is_none());
        assert!(filter
            .tags
            .get(&"_invalid".to_string().into_bytes())
            .is_none());
        assert!(filter.tags.get(&"b".to_string().into_bytes()).is_none());
        // set tag
        filter.set_tags(HashMap::from([
            (
                "t".to_string(),
                vec![
                    "ab".to_string(),
                    "ab".to_string(),
                    "cd".to_string(),
                    "12".to_string(),
                ],
            ),
            (
                "g".to_string(),
                vec!["ab".to_string(), "cd".to_string(), "12".to_string()],
            ),
        ]));
        assert_eq!(
            &filter.tags.get(&"t".to_string().into_bytes()),
            &Some(&tags)
        );
        assert_eq!(
            &filter.tags.get(&"g".to_string().into_bytes()),
            &Some(&tags)
        );
        assert!(filter.tags.get(&"d".to_string().into_bytes()).is_none());

        // search
        let note = r###"
        {
            "keywords": ["abc", "def"]
          }
        "###;
        let filter: Filter = serde_json::from_str(note)?;
        assert_eq!(filter.search, Some("abc def".to_string()));

        let note = r###"
        {
            "keywords": ["abc", "def"],
            "search": "t"
          }
        "###;
        let filter: Filter = serde_json::from_str(note)?;
        assert_eq!(filter.search, Some("t".to_string()));

        // invalid
        let note = r###"
        {
            "#g": ["ab", "cd", 12]
          }
        "###;
        let filter: Result<Filter, _> = serde_json::from_str(note);
        assert!(filter.is_err());

        let note = r###"
        {
            "#e": ["ab"],
            "#p": ["ab"]
          }
        "###;
        let filter = Filter::from_str(note);
        assert!(filter.is_err());

        let note = r###"
        {
            "#e": ["0000000000000000000000000000000000000000000000000000000000000000"],
            "#p": ["0000000000000000000000000000000000000000000000000000000000000000"]
          }
        "###;
        let filter = Filter::from_str(note)?;
        assert!(filter
            .tags
            .get(&b"e".to_vec())
            .unwrap()
            .contains(&vec![0u8; 32]));
        let filter = Filter::from_str(note)?;
        assert!(filter
            .tags
            .get(&b"p".to_vec())
            .unwrap()
            .contains(&vec![0u8; 32]));
        Ok(())
    }

    fn check_match(
        s: &str,
        matched: bool,
        event: &Event,
        archived: &ArchivedEventIndex,
    ) -> Result<()> {
        let filter: Filter = serde_json::from_str(s)?;
        if matched {
            assert!(filter.r#match(event.index()));
            assert!(filter.match_archived(archived));
        } else {
            assert!(!filter.r#match(event.index()));
            assert!(!filter.match_archived(archived));
        }
        Ok(())
    }

    #[test]
    fn match_event() -> Result<()> {
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"], ["t", "db"], ["subject", "db"]]
          }
        "#;
        let event: Event = serde_json::from_str(note)?;
        let bytes = event.index().to_bytes()?;
        let archived = EventIndex::from_zeroes(&bytes)?;

        check_match(
            r###"
        {
        }
        "###,
            true,
            &event,
            archived,
        )?;

        check_match(
            r###"
        {
            "ids": ["332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d", "0000000000000000000000000000000000000000000000000000000000000000"],
            "authors": ["7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef", "0000000000000000000000000000000000000000000000000000000000000000"],
            "kind": [1, 2],
            "#t": ["nostr", "other"],
            "#subject": ["db", "other"],
            "since": 1680690000,
            "util": 2680690000
        }
        "###,
            true,
            &event,
            archived,
        )?;

        check_match(
            r###"
        {
            "#t": ["other"]
        }
        "###,
            false,
            &event,
            archived,
        )?;

        check_match(
            r###"
        {
            "#t": ["nostr"],
            "#r": ["nostr"]
        }
        "###,
            false,
            &event,
            archived,
        )?;

        check_match(
            r###"
        {
            "ids": ["332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d"]
        }
        "###,
            true,
            &event,
            archived,
        )?;

        check_match(
            r###"
        {
            "ids": ["abababababababababababababababababababababababababababababababab"]
        }
        "###,
            false,
            &event,
            archived,
        )?;

        Ok(())
    }

    #[test]
    fn tag_contains() -> Result<()> {
        let note = r#"
        {
            "content": "Good morning everyone 😃",
            "created_at": 1680690006,
            "id": "332747c0fab8a1a92def4b0937e177be6df4382ce6dd7724f86dc4710b7d4d7d",
            "kind": 1,
            "pubkey": "7abf57d516b1ff7308ca3bd5650ea6a4674d469c7c5057b1d005fb13d218bfef",
            "sig": "ef4ff4f69ac387239eb1401fb07d7a44a5d5d57127e0dc3466a0403cf7d5486b668608ebfcbe9ff1f8d3b5d710545999fe08ee767284ec0b474e4cf92537678f",
            "tags": [["t", "nostr"], ["t", "db"], ["r", "db"]]
          }
        "#;
        let event: Event = serde_json::from_str(note)?;
        assert!(Filter::tag_contains(
            event.index().tags(),
            &"t".to_string().into_bytes(),
            &vec!["nostr".to_string().into_bytes()].into()
        ));
        assert!(Filter::tag_contains(
            event.index().tags(),
            &"t".to_string().into_bytes(),
            &vec![
                "nostr".to_string().into_bytes(),
                "other".to_string().into_bytes()
            ]
            .into()
        ));

        assert!(!Filter::tag_contains(
            event.index().tags(),
            &"t".to_string().into_bytes(),
            &vec![
                "nostr1".to_string().into_bytes(),
                "other".to_string().into_bytes()
            ]
            .into()
        ));
        Ok(())
    }
}
```

---

### key.rs

**Size:** 7244 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use crate::error::Error;
use nostr_kv::scanner::TimeKey;

// a separator for compare
pub const VIEW_KEY_SEP: [u8; 1] = [0];
// a separator for compare
pub fn concat_sep<K, I>(one: K, two: I) -> Vec<u8>
where
    K: AsRef<[u8]>,
    I: AsRef<[u8]>,
{
    [one.as_ref(), &VIEW_KEY_SEP, two.as_ref()].concat()
}

pub fn concat<K, I>(one: K, two: I) -> Vec<u8>
where
    K: AsRef<[u8]>,
    I: AsRef<[u8]>,
{
    [one.as_ref(), two.as_ref()].concat()
}

pub struct IndexKey {
    time: u64,
    uid: u64,
}

impl IndexKey {
    pub fn encode_time(time: u64) -> Vec<u8> {
        time.to_be_bytes().to_vec()
    }

    pub fn encode_id<K: AsRef<[u8]>>(id: K, time: u64) -> Vec<u8> {
        [id.as_ref(), &time.to_be_bytes()[..]].concat()
    }

    pub fn encode_kind(kind: u16, time: u64) -> Vec<u8> {
        [&kind.to_be_bytes()[..], &time.to_be_bytes()[..]].concat()
    }

    pub fn encode_pubkey<P: AsRef<[u8]>>(pubkey: P, time: u64) -> Vec<u8> {
        [pubkey.as_ref(), &time.to_be_bytes()[..]].concat()
    }

    pub fn encode_pubkey_kind<P: AsRef<[u8]>>(pubkey: P, kind: u16, time: u64) -> Vec<u8> {
        [
            pubkey.as_ref(),
            &kind.to_be_bytes()[..],
            &time.to_be_bytes()[..],
        ]
        .concat()
    }

    pub fn encode_tag<TK: AsRef<[u8]>, TV: AsRef<[u8]>>(
        tag_key: TK,
        tag_val: TV,
        time: u64,
    ) -> Vec<u8> {
        Self::encode_tag1(concat_sep(tag_key, tag_val), time)
    }

    fn encode_tag1<T: AsRef<[u8]>>(tag: T, time: u64) -> Vec<u8> {
        [tag.as_ref(), &VIEW_KEY_SEP, &time.to_be_bytes()[..]].concat()
    }

    pub fn encode_word<P: AsRef<[u8]>>(word: P, time: u64) -> Vec<u8> {
        [word.as_ref(), &VIEW_KEY_SEP, &time.to_be_bytes()[..]].concat()
    }

    pub fn from(key: &[u8], uid: &[u8]) -> Result<Self, Error> {
        let time: u64 = u64::from_be_bytes(key[(key.len() - 8)..].try_into()?);
        let uid: u64 = u64::from_be_bytes(uid[..8].try_into()?);
        Ok(Self { time, uid })
    }

    pub fn uid(&self) -> u64 {
        self.uid
    }
}

impl TimeKey for IndexKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time()
            .cmp(&other.time())
            .then_with(|| self.uid().cmp(&other.uid()))
    }

    fn change_time(&self, key: &[u8], time: u64) -> Vec<u8> {
        let pos = key.len() - 8;
        [&key[0..pos], &time.to_be_bytes()[..]].concat()
    }

    fn time(&self) -> u64 {
        self.time
    }
}

pub fn u64_to_ver(num: u64) -> Vec<u8> {
    num.to_be_bytes().to_vec()
}

pub fn u16_to_ver(num: u16) -> Vec<u8> {
    num.to_be_bytes().to_vec()
}

// Replaceable Events [NIP-16](https://nips.be/16)
// Parameterized Replaceable Events [NIP-33](https://nips.be/33)
pub fn encode_replace_key(kind: u16, pubkey: &[u8; 32], tags: &[Vec<String>]) -> Option<Vec<u8>> {
    if kind == 0 || kind == 3 || kind == 41 || (10_000..20_000).contains(&kind) {
        let k = u16_to_ver(kind);
        let p: &[u8] = pubkey.as_ref();
        Some([p, &k[..]].concat())
    } else if (30_000..40_000).contains(&kind) {
        let k = u16_to_ver(kind);
        let p: &[u8] = pubkey.as_ref();
        let tag = tags
            .get(0)
            .map(|tag| {
                if tag.len() > 1 && tag[0] == "d" {
                    tag.get(1).unwrap().clone()
                } else {
                    "".to_owned()
                }
            })
            .unwrap_or_default();
        Some([p, &k[..], tag.as_bytes()].concat())
    } else {
        None
    }
}
type ReplaceKey<'a> = (&'a [u8], u16, &'a [u8], u64);
#[allow(unused)]
pub fn decode_replace_key<'a>(val: &'a [u8], time: &'a [u8]) -> Result<ReplaceKey<'a>, Error> {
    let len = val.len();
    if len < 32 + 2 {
        Err(Error::InvalidLength)
    } else {
        let pubkey = &val[0..32];
        let kind = u16::from_be_bytes(val[32..34].try_into()?);
        let tag = &val[34..];
        let time = u64::from_be_bytes(time.try_into()?);
        Ok((pubkey, kind, tag, time))
    }
}

// pad '0' at beginning if has not enough length
#[allow(unused)]
pub fn pad_start(id: &Vec<u8>, len: usize) -> Vec<u8> {
    let num = len as i32 - id.len() as i32;
    match num.cmp(&0) {
        std::cmp::Ordering::Less => id[0..len].to_vec(),
        std::cmp::Ordering::Equal => id.clone(),
        std::cmp::Ordering::Greater => {
            let num = num as usize;
            let mut ret = vec![0; len];
            ret[num..].copy_from_slice(id);
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn pad() {
        assert_eq!(
            pad_start(&vec![1, 2, 3], 32),
            [vec![0u8; 29], vec![1, 2, 3]].concat()
        );
        assert_eq!(pad_start(&vec![1; 33], 32), vec![1; 32]);
        assert_eq!(pad_start(&vec![2; 32], 32), vec![2; 32]);
    }

    #[test]
    fn index_key() -> Result<()> {
        let time = 20u64;
        let id = pad_start(&vec![1, 2, 3], 32);
        let kind = 10u16;
        let pubkey = vec![1; 32];
        let tag_key = "d";
        let tag_val = "m";
        let uid_num = 2u64;
        let uid: Vec<u8> = uid_num.to_be_bytes().to_vec();
        let ind = IndexKey::from(&IndexKey::encode_id(id, time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);

        let ind = IndexKey::from(&IndexKey::encode_time(time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);

        let ind = IndexKey::from(&IndexKey::encode_kind(kind, time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);
        let ind = IndexKey::from(&IndexKey::encode_pubkey(&pubkey, time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);

        let ind = IndexKey::from(&IndexKey::encode_pubkey_kind(&pubkey, kind, time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);

        let ind = IndexKey::from(&IndexKey::encode_tag(tag_key, tag_val, time), &uid)?;
        assert_eq!(ind.uid, uid_num);
        assert_eq!(ind.time, time);

        Ok(())
    }

    #[test]
    fn replace_key() {
        let tags = vec![vec!["d".to_owned(), "m".to_owned()]];
        let pubkey = [1u8; 32];
        let time = u64_to_ver(10);
        let empty: Vec<u8> = vec![];

        assert!(encode_replace_key(1, &pubkey, &tags).is_none());
        assert!(decode_replace_key(&[1], &time).is_err());

        let k = encode_replace_key(0, &pubkey, &tags).unwrap();
        let r = decode_replace_key(&k, &time).unwrap();
        assert_eq!(r.0, &pubkey);
        assert_eq!(r.1, 0);
        assert_eq!(r.2, empty);
        assert_eq!(r.3, 10);

        let k = encode_replace_key(10001, &pubkey, &tags).unwrap();
        let r = decode_replace_key(&k, &time).unwrap();
        assert_eq!(r.0, &pubkey);
        assert_eq!(r.1, 10001);
        assert_eq!(r.2, empty);
        assert_eq!(r.3, 10);

        let k = encode_replace_key(30001, &pubkey, &tags).unwrap();
        let r = decode_replace_key(&k, &time).unwrap();
        assert_eq!(r.0, &pubkey);
        assert_eq!(r.1, 30001);
        assert_eq!(r.2, "m".as_bytes());
        assert_eq!(r.3, 10);
    }
}
```

---

### lib.rs

**Size:** 1031 bytes | **Modified:** 2025-11-22 21:18:16

```rust
//! Nostr event database

mod db;
mod error;
mod event;
mod filter;
mod key;
pub use secp256k1;

pub use {
    db::CheckEventResult, db::Db, db::Iter, error::Error, event::now, event::ArchivedEventIndex,
    event::Event, event::EventIndex, event::FromEventData, filter::Filter, filter::SortList,
};

pub use nostr_kv as kv;

/// Stats of query
#[derive(Debug, Clone)]
pub struct Stats {
    pub scan_index: u64,
    pub get_data: u64,
    pub get_index: u64,
}

#[cfg(feature = "search")]
use charabia::Segment;

#[cfg(feature = "search")]
/// segment keywords by charabia
pub fn segment(content: &str) -> Vec<Vec<u8>> {
    let iter = content.segment_str();
    let mut words = iter
        .filter_map(|s| {
            let s = s.to_lowercase();
            let bytes = s.as_bytes();
            // limit size
            if bytes.len() < 255 {
                Some(bytes.to_vec())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    words.sort();
    words.dedup();
    words
}
```

---


---
*Generated by code2prompt.sh on 2026-01-13 15:20:01*
