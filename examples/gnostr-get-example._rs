use std::io::Read;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use gnostr_bins::get_blockheight;
use reqwest::Url;
use tokio::runtime::Runtime;

//use ureq::get;

const URL: &str = "https://mempool.space/api/blocks/tip/height";

fn main() {
    let n = 1;
    {
        let start = Instant::now();
        let res = blocking(n);
        println!("blocking {:?} {} bytes", start.elapsed(), res);
    }
    {
        let start = Instant::now();
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(non_blocking(n));
        println!("async    {:?} {} bytes", start.elapsed(), res);
    }
}

fn blocking(n: usize) -> usize {
    (0..n)
        .into_iter()
        .map(|_| {
            std::thread::spawn(|| {
                let mut body = ureq::get(URL).call().expect("REASON").into_reader();
                let mut buf = Vec::new();
                body.read_to_end(&mut buf).unwrap();
                // print block count from mempool.space or panic
                let text = match std::str::from_utf8(&buf) {
                    Ok(s) => s,
                    Err(_) => panic!("Invalid ASCII data"),
                };
                println!("{}", text);
                buf.len()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|it| it.join().unwrap())
        .sum()
}

async fn non_blocking(n: usize) -> usize {
    let tasks = (0..n)
        .into_iter()
        .map(|_| {
            tokio::spawn(async move {
                let since_the_epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("get millis error");
                let seconds = since_the_epoch.as_secs();
                let subsec_millis = since_the_epoch.subsec_millis() as u64;
                let now_millis = seconds * 1000 + subsec_millis;
                //println!("now millis: {}", seconds * 1000 + subsec_millis);

                let _ = get_blockheight();
                let url = Url::parse(URL).unwrap();
                let mut res = reqwest::blocking::get(url).unwrap();

                let mut tmp_string = String::new();
                res.read_to_string(&mut tmp_string).unwrap();
                //println!("{}", format!("{:?}", res));
                let tmp_u64 = tmp_string.parse::<u64>().unwrap_or(0);
                println!("{}", format!("{:?}", tmp_u64));

                //TODO:impl gnostr-weeble_millis
                //let weeble = now_millis as f64 / tmp_u64 as f64;
                //let weeble = seconds as f64 / tmp_u64 as f64;
                //println!("{}", format!("{}", weeble.floor()));

                let body = reqwest::get(URL).await.unwrap().bytes();
                body.await.unwrap().len()
                // print block count from mempool.space or panic
                //let text = match std::str::from_utf8(&body) {
                //    Ok(s) => s,
                //    Err(_) => panic!("Invalid ASCII data"),
                //};
                //println!("{}", text);
            })
        })
        .collect::<Vec<_>>();

    let mut res = 0;
    for task in tasks {
        res += task.await.unwrap();
    }
    res
}
