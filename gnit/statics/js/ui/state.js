const VM_FRIENDS = "friends" // mine + only events that are from my contacts
const VM_NOTIFICATIONS = "notifications" // reactions & replys
const VM_DM = "dm" // all events of KIND_DM aimmed at user
const VM_DM_THREAD = "dmthread" // all events from a user of KIND_DM
const VM_THREAD = "thread" // all events in response to target event
const VM_USER = "user" // all events by pubkey
const VM_SETTINGS = "settings"
const VM_NIP34 = "nip34-view-friends" // NIP-34 events from followed profiles
const VM_GNOSTR = "gnostr" // All NIP-34 events
const VM_NIP34_DETAIL = "nip34-detail" // Detailed view of a NIP-34 repository

const VM_SEARCH = "search" // Search results

const VIEW_NAMES = {}
VIEW_NAMES[VM_FRIENDS] = "home"
VIEW_NAMES[VM_NOTIFICATIONS] = "notifications"
VIEW_NAMES[VM_DM] = "messages"
VIEW_NAMES[VM_DM_THREAD] = "dm"
VIEW_NAMES[VM_USER] = "profile"
VIEW_NAMES[VM_THREAD] = "thread"
VIEW_NAMES[VM_SETTINGS] = "settings"
VIEW_NAMES[VM_NIP34] = "nip-34"
VIEW_NAMES[VM_GNOSTR] = "gnostr"
VIEW_NAMES[VM_NIP34_DETAIL] = "repository-details"
VIEW_NAMES[VM_SEARCH] = "Search"

function view_get_timeline_el() {
  return find_node("#timeline")
}

// TODO clean up popstate listener (move to init method or such)
window.addEventListener("popstate", function (event) {
  if (event.state && event.state.mode) {
    // Update the timeline mode.
    // Pass pushState=false to avoid adding another state to the history
    view_timeline_apply_mode(GNOSTR, event.state.mode, event.state.opts, false)
  }
})

function view_timeline_apply_mode(model, mode, opts = {}, push_state = true) {
  console.log(`view_timeline_apply_mode called. Mode: ${mode}, Opts:`, opts)
  let xs
  const { pubkey, thread_id } = opts
  const el = view_get_timeline_el()
  const now = new Date().getTime()

  if (opts.hide_replys == undefined) {
    opts.hide_replys = el.dataset.hideReplys == "true"
  }

  // Don't do anything if we are already here
  if (el.dataset.mode == mode) {
    switch (mode) {
      case VM_FRIENDS:
        if ((el.dataset.hideReplys == "true") == opts.hide_replys) return
        push_state = false
        break
      case VM_DM_THREAD:
      case VM_USER:
        if (el.dataset.pubkey == opts.pubkey) return
        break
      case VM_THREAD:
        if (el.dataset.threadId == thread_id) return
        break
      case VM_NIP34_DETAIL:
        if (el.dataset.repoId == opts.repo_id) return
        break
      default:
        return
    }
  }

  // Fetch history for certain views
  if (mode == VM_THREAD) {
    view_show_spinner(true)
    const clicked_event = model.all_events[thread_id]
    if (clicked_event && clicked_event.kind >= KIND_REPO_ANNOUNCE && clicked_event.kind <= KIND_REPO_STATUS_DRAFT) {
      const repo_id_tag = clicked_event.tags.find((tag) => tag[0] === "a")
      if (repo_id_tag) {
        const repo_id = repo_id_tag[1]
        const parts = repo_id.split(":")
        if (parts.length === 3) {
          const kind = parseInt(parts[0])
          const pubkey = parts[1]
          const d_tag = parts[2]
          if (kind === KIND_REPO_ANNOUNCE) {
            model.pool.subscribe(`repo:${thread_id}`, [
              {
                kinds: [kind],
                authors: [pubkey],
                "#d": [d_tag],
              },
            ])
          }
        }
      }
    }
    fetch_thread_history(thread_id, model.pool)
  }
  if (mode == VM_USER && pubkey && pubkey != model.pubkey) {
    view_show_spinner(true)
    fetch_profile(pubkey, model.pool)
  }
  if (mode == VM_NOTIFICATIONS) {
    reset_notifications(model)
  }
  if (mode == VM_NIP34 || mode == VM_GNOSTR) {
    view_show_spinner(true)
    // NIP-34 events are already fetched as part of PUBLIC_KINDS,
    // so no special fetch is needed here beyond what's done for friends.
  }
  if (mode == VM_NIP34_DETAIL) {
    console.log(`view_timeline_apply_mode: Entering VM_NIP34_DETAIL for repo_id: ${opts.repo_id}`)
  }

  const names = VIEW_NAMES
  let name = names[mode]
  let profile

  // Push a new state to the browser history stack
  if (push_state) {
    let pieces = [name.toLowerCase()]
    switch (mode) {
      case VM_FRIENDS:
        pieces = []
        break
      case VM_THREAD:
        pieces.push(thread_id)
        break
      case VM_USER:
      case VM_DM_THREAD:
      case VM_GNOSTR: // Add for nip34-global
        pieces.push(pubkey) // This should be empty for global
        break
      case VM_NIP34_DETAIL:
        pieces.push(opts.repo_id)
        break
    }
    window.history.pushState({ mode, opts }, "", "/" + pieces.join("/"))
  }

  el.dataset.mode = mode
  delete el.dataset.threadId
  delete el.dataset.pubkey
  delete el.dataset.repoId
  switch (mode) {
    case VM_FRIENDS:
      el.dataset.hideReplys = opts.hide_replys
      name = "Home"
      break
    case VM_THREAD:
      el.dataset.threadId = thread_id
      break
    case VM_USER:
    case VM_DM_THREAD:
      profile = model_get_profile(model, pubkey)
      name = fmt_name(profile)
      el.dataset.pubkey = pubkey
      break
    case VM_NIP34_DETAIL:
      el.dataset.repoId = opts.repo_id
      //finger print
      name =
        "gnostr://" +
        (opts.repo_name || opts.repo_id.slice(5 + 1, 5 + 1 + 8) + "/" + opts.repo_id.slice(5 + 1 + 64 + 1, 100))
      break
  }

  // Do some visual updates
  find_node("#show-more").classList.add("hide")
  // Capitalize the first letter of the name for display, unless it's a special case
  if (mode !== VM_USER && mode !== VM_DM_THREAD && mode !== VM_NIP34_DETAIL) {
    name = name.charAt(0).toUpperCase() + name.slice(1)
  }
  find_node("#view header > label").innerText = name
  view_update_navs(mode)
  find_node("#view [role='profile-info']").classList.toggle("hide", mode != VM_USER)
  const timeline_el = find_node("#timeline")
  timeline_el.classList.toggle("reverse", mode == VM_DM_THREAD)
  timeline_el.classList.toggle("hide", mode == VM_SETTINGS || mode == VM_DM)
  find_node("#settings").classList.toggle("hide", mode != VM_SETTINGS)
  find_node("#dms").classList.toggle("hide", mode != VM_DM)
  find_node("#dm-post").classList.toggle("hide", mode != VM_DM_THREAD)
  find_node("#new-note-mobile").classList.toggle("hide", mode == VM_DM_THREAD)
  find_node("#header-tools button[action='mark-all-read']").classList.toggle("hide", mode != VM_DM)

  // Show/hide different profile image in header
  const show_mypfp = mode != VM_DM_THREAD && mode != VM_USER
  const el_their_pfp = find_node("#view header img.pfp[role='their-pfp']")
  el_their_pfp.classList.toggle("hide", show_mypfp)
  find_node("#view header img.pfp[role='my-pfp']").classList.toggle("hide", !show_mypfp)

  view_timeline_refresh(model, mode, opts)

  switch (mode) {
    case VM_DM_THREAD:
      decrypt_dms(model)
      model_dm_seen(model, pubkey)
      el_their_pfp.src = get_profile_pic(profile)
      el_their_pfp.dataset.pubkey = pubkey
      break
    case VM_DM:
      model.dms_need_redraw = true
      view_show_spinner(true)
      view_set_show_count(0, true, true)
      //decrypt_dms(model);
      //view_dm_update(model);
      break
    case VM_SETTINGS:
      view_show_spinner(false)
      view_set_show_count(0, true, true)
      break
    case VM_USER:
      el_their_pfp.src = get_profile_pic(profile)
      el_their_pfp.dataset.pubkey = pubkey
      view_update_profile(model, pubkey)
      break
    case VM_NIP34_DETAIL:
      view_show_spinner(true)
      fetch_repo_events(opts.repo_id, model.pool)
      break
  }
  return mode
}

/* view_timeline_refresh is a hack for redrawing the events in order
 */
function view_timeline_refresh(model, mode, opts = {}) {
  console.log(`view_timeline_refresh called. Mode: ${mode}, Opts:`, opts)
  const el = view_get_timeline_el()
  if (!mode) {
    mode = el.dataset.mode
    opts.thread_id = el.dataset.threadId
    opts.pubkey = el.dataset.pubkey
    opts.repo_id = el.dataset.repoId
    opts.hide_replys = el.dataset.hideReplys == "true"
  }
  if (!opts.is_showing_more) {
    el.innerHTML = ""
  }

  let evs = model_events_arr(model)

  if (mode == VM_THREAD) {
    const clicked_event = model.all_events[opts.thread_id]
    if (clicked_event) {
      const root_id = clicked_event.refs.root || clicked_event.id
      const root_event = model.all_events[root_id]

      if (root_event) {
        let announcement_event = null
        if (root_event.kind === KIND_REPO_STATE_ANNOUNCE) {
          const d_tag = root_event.tags.find((tag) => tag[0] === "d")
          if (d_tag) {
            const repo_id = d_tag[1]
            for (const key in model.all_events) {
              const ev = model.all_events[key]
              if (ev.kind === KIND_REPO_ANNOUNCE && ev.tags.find((t) => t[0] === "d" && t[1] === repo_id)) {
                announcement_event = ev
                break
              }
            }
          }
        }

        const replies = []
        for (const ev of evs) {
          if (ev.id !== root_id && event_refs_event(ev, { id: root_id })) {
            replies.push(ev)
          }
        }
        replies.sort((a, b) => a.created_at - b.created_at)

        const fragment = new DocumentFragment()
        if (announcement_event) {
          const el = view_render_event(model, announcement_event)
          if (el) fragment.appendChild(el)
        }
        const root_el = view_render_event(model, root_event)
        if (root_el) fragment.appendChild(root_el)

        for (const reply of replies) {
          const reply_el = view_render_event(model, reply)
          if (reply_el) fragment.appendChild(reply_el)
        }
        el.appendChild(fragment)
      }
    }
  } else {
    // Existing logic for other views
    if (mode == VM_NIP34_DETAIL || mode == VM_GNOSTR || mode == VM_NIP34) {
      evs.sort((a, b) => b.created_at - a.created_at)
    } else {
      evs.reverse()
    }

    const fragment = new DocumentFragment()
    let count = 0
    const limit = 50
    for (let i = 0; i < evs.length && count < limit; i++) {
      const ev = evs[i]
      if (!view_mode_contains_event(model, ev, mode, opts)) continue
      let ev_el = model.elements[ev.id]
      if (!ev_el) {
        ev_el = view_render_event(model, ev)
        if (!ev_el) continue
      }
      fragment.appendChild(ev_el)
      count++
    }
    if (count > 0) {
      el.append(fragment)
    }
    const is_more_mode = mode == VM_FRIENDS || mode == VM_NOTIFICATIONS || mode == VM_NIP34_DETAIL || mode == VM_GNOSTR
    if (is_more_mode && evs.length > limit) {
      find_node("#show-more").classList.remove("hide")
    } else {
      find_node("#show-more").classList.add("hide")
    }
  }

  view_set_show_count(0)
  view_timeline_update_timestamps()
  view_show_spinner(false)
}

function view_update_navs(mode) {
  find_nodes("nav.nav button[data-view]").forEach((el) => {
    el.classList.toggle("active", el.dataset.view == mode)
  })
}

function view_show_spinner(show = true) {
  find_node("#view .loading-events").classList.toggle("hide", !show)
}

function view_get_el_opts(el) {
  const mode = el.dataset.mode
  return {
    thread_id: el.dataset.threadId,
    pubkey: el.dataset.pubkey,
    hide_replys: mode == VM_FRIENDS && el.dataset.hideReplys == "true",
  }
}

/* view_timeline_update iterates through invalidated event ids and updates the
 * state of the timeline and other factors such as notifications, etc.
 */
function view_timeline_update(model) {
  const el = view_get_timeline_el()
  const mode = el.dataset.mode
  const opts = view_get_el_opts(el)
  let count = 0
  let ncount = 0
  let decrypted = false
  const latest_ev = el.firstChild ? model.all_events[el.firstChild.id.slice(2)] : undefined
  const left_overs = []
  while (model.invalidated.length > 0 && count < 500) {
    var evid = model.invalidated.pop()

    // Remove deleted events first
    if (model_is_event_deleted(model, evid)) {
      let x = model.elements[evid]
      if (x && x.parentElement) {
        x.parentElement.removeChild(x)
        delete model.elements[evid]
      }
      continue
    }

    // Skip non-renderables
    var ev = model.all_events[evid]
    if (!event_is_renderable(ev)) {
      continue
    }

    // Re-render content of a decrypted dm
    if (ev.kind == KIND_DM && model.elements[evid]) {
      rerender_dm(model, ev, model.elements[evid])
      decrypted = true
      continue
    }

    // Put it back on the stack to re-render if it's not ready.
    if (!view_render_event(model, ev)) {
      left_overs.push(evid)
      continue
    }

    // Increase notification count if needed
    if (event_refs_pubkey(ev, model.pubkey) && ev.created_at > model.notifications.last_viewed) {
      ncount++
    }

    // If the new element is newer than the latest & is viewable then
    // we want to increase the count of how many to add to view
    if (event_cmp_created(ev, latest_ev) >= 0 && view_mode_contains_event(model, ev, mode, opts)) {
      count++
    }
  }
  model.invalidated = model.invalidated.concat(left_overs)

  // If there are new things to show on our current view lets do it
  if (count > 0) {
    if (!latest_ev || mode == VM_DM_THREAD) {
      view_timeline_show_new(model)
    }
    if (mode == VM_DM_THREAD) {
      model_mark_dms_seen(model, opts.pubkey)
      view_dm_update(model)
    }
    view_set_show_count(count, true, false)
  }
  // Update notification markers and count
  if (ncount > 0) {
    //log_debug(`new notis ${ncount}`);
    model.notifications.count += ncount
  }
  // Update the dms list view
  if (decrypted) {
    view_dm_update(model)
  }
}

function view_set_show_count(count, add = false, hide = false) {
  const show_el = find_node("#show-new")
  const num_el = find_node("#show-new span", show_el)
  if (!num_el) {
    return
  }

  if (num_el.innerText !== null) {
    if (add) {
      count += parseInt(num_el.innerText || 0)
    }
    num_el.innerText = count

    // You might also need to check for the count being zero to hide it
    if (count === 0) {
      num_el.style.display = "none"
    } else {
      num_el.style.display = "block" // Or whatever default display is
    }
  }
  show_el.classList.toggle("hide", hide || count <= 0)
}

function view_timeline_show_new(model) {
  const el = view_get_timeline_el()
  const mode = el.dataset.mode
  const opts = view_get_el_opts(el)
  let latest_evid = el.firstChild ? el.firstChild.id.slice(2) : undefined
  if (mode == VM_THREAD) {
    latest_evid = el.lastElementChild ? el.lastElementChild.id.slice(2) : undefined
  }

  let count = 0
  const evs = model_events_arr(model)
  const fragment = new DocumentFragment()
  for (let i = evs.length - 1; i >= 0 && count < 500; i--) {
    const ev = evs[i]
    if (latest_evid && ev.id == latest_evid) {
      break
    }
    if (!view_mode_contains_event(model, ev, mode, opts)) continue
    let ev_el = model.elements[ev.id]
    if (!ev_el) continue
    fragment.appendChild(ev_el)
    count++
  }
  if (count > 0) {
    if (mode == VM_THREAD) {
      el.appendChild(fragment)
    } else {
      el.prepend(fragment)
    }
    view_show_spinner(false)
    if (mode == VM_NOTIFICATIONS) {
      reset_notifications(model)
    }
  }
  view_set_show_count(-count, true)
  view_timeline_update_timestamps()
  if (mode == VM_DM_THREAD) decrypt_dms(model)
}

function view_timeline_show_more(model) {
  const el = view_get_timeline_el()
  const mode = el.dataset.mode
  const opts = view_get_el_opts(el)
  const oldest_evid = el.lastElementChild ? el.lastElementChild.id.slice(2) : undefined
  const oldest = model.all_events[oldest_evid]
  const evs = model_events_arr(model)
  const fragment = new DocumentFragment()
  let i = arr_bsearch(evs, oldest, (a, b) => {
    if (a.id == b.id || a.created_at == b.created_at) return 0
    if (a.created_at > b.created_at) return 1
    return -1
  })
  const limit = 200
  let count = 0
  for (; i >= 0 && count < limit; i--) {
    const ev = evs[i]
    if (!view_mode_contains_event(model, ev, mode, opts)) continue
    let ev_el = model.elements[ev.id]
    if (!ev_el || ev_el.parentElement) continue
    fragment.appendChild(ev_el)
    count++
  }
  if (count > 0) {
    el.append(fragment)
  }
  if (count < limit) {
    // No more to show, hide the button
    find_node("#show-more").classList.add("hide")
  }
  view_timeline_update_timestamps()
}

function view_timeline_show_more(model) {
  const el = view_get_timeline_el()
  const mode = el.dataset.mode
  const opts = view_get_el_opts(el)
  const oldest_evid = el.lastElementChild ? el.lastElementChild.id.slice(2) : undefined
  const oldest = model.all_events[oldest_evid]
  const evs = model_events_arr(model)
  const fragment = new DocumentFragment()
  let i = arr_bsearch(evs, oldest, (a, b) => {
    if (a.id == b.id || a.created_at == b.created_at) return 0
    if (a.created_at > b.created_at) return 1
    return -1
  })
  const limit = 200
  let count = 0
  for (; i >= 0 && count < limit; i--) {
    const ev = evs[i]
    if (!view_mode_contains_event(model, ev, mode, opts)) continue
    let ev_el = model.elements[ev.id]
    if (!ev_el || ev_el.parentElement) continue
    fragment.appendChild(ev_el)
    count++
  }
  if (count > 0) {
    el.append(fragment)
  }
  if (count < limit) {
    // No more to show, hide the button
    find_node("#show-more").classList.add("hide")
  }
  view_timeline_update_timestamps()
}

function show_more_nip34_events(model) {
  console.log("show_more_nip34_events called.")
  const el = view_get_timeline_el()
  const repo_id = el.dataset.repoId
  if (!repo_id) {
    console.error("show_more_nip34_events: No repo_id found in dataset.")
    return
  }

  const oldest_event_el = el.lastElementChild // Assuming chronological order, last element is oldest
  if (!oldest_event_el) {
    console.log("show_more_nip34_events: No oldest event found in timeline.")
    return
  }

  const oldest_evid = oldest_event_el.id.slice(2)
  const oldest_event = model.all_events[oldest_evid]
  if (!oldest_event) {
    console.error(`show_more_nip34_events: Oldest event ${oldest_evid} not found in model.`)
    return
  }
  const until = oldest_event.created_at

  console.log(`show_more_nip34_events: Fetching more NIP-34 events for repo_id: ${repo_id} until: ${until}`)
  fetch_repo_events(repo_id, model.pool, until)

  // Temporarily hide the show-more button until new events are loaded
  find_node("#show-more").classList.add("hide")
}

function view_render_event(model, ev, force = false) {
  if (model.elements[ev.id] && !force) return model.elements[ev.id]
  const html = render_event(model, ev, {})
  if (html == "") {
    //log_debug(`failed to render ${ev.id}`);
    return
  }
  const div = document.createElement("div")
  div.innerHTML = html
  const el = div.firstChild
  model.elements[ev.id] = el
  const pfp = find_node("img.pfp", el)
  if (pfp) pfp.addEventListener("error", onerror_pfp)
  return el
}

function view_timeline_update_profiles(model, pubkey) {
  const el = view_get_timeline_el()
  const p = model_get_profile(model, pubkey)
  const name = fmt_name(p)
  const pic = get_profile_pic(p)
  for (const evid in model.elements) {
    // XXX if possible update profile pics in a smarter way
    // this may be perhaps a micro optimization tho
    update_el_profile(model.elements[evid], pubkey, name, pic)
  }
  // Update the profile view if it's active
  if (el.dataset.pubkey == pubkey) {
    const mode = el.dataset.mode
    switch (mode) {
      case VM_USER:
        view_update_profile(model, pubkey)
      case VM_DM_THREAD:
        find_node("#view header > label").innerText = name
    }
  }
  // Update dm's section since they are not in our view, dm's themselves will
  // be caught by the process above.
  update_el_profile(find_node("#dms"), pubkey, name, pic)
  update_el_profile(find_node("#view header"), pubkey, name, pic)
}

function update_el_profile(el, pubkey, name, pic) {
  if (!el) return
  find_nodes(`.username[data-pubkey='${pubkey}']`, el).forEach((el) => {
    el.innerText = name
  })
  find_nodes(`img[data-pubkey='${pubkey}']`, el).forEach((el) => {
    el.src = pic
    el.title = name
  })
}

function view_timeline_update_timestamps() {
  // TODO only update elements that are fresh and are in DOM
  const el = view_get_timeline_el()
  let xs = el.querySelectorAll(".timestamp")
  let now = new Date().getTime()
  for (const x of xs) {
    let t = parseInt(x.dataset.timestamp)
    x.innerText = fmt_since_str(now, t * 1000)
  }
}

///

function render_dummy_events(container_el, count) {
  const fragment = new DocumentFragment()
  for (let i = 0; i < count; i++) {
    const dummy_el = document.createElement("div")
    dummy_el.classList.add("dummy-event", "event-base") // Add event-base for consistent styling
    dummy_el.innerHTML = `
            <div class="post-info">
                <img class="pfp skeleton" style="width:40px;height:40px;border-radius:50%;" src="/src/images/no-user.svg"></img>
                <div class="post-identifiers">
                    <div class="author skeleton" style="width:100px;height:12px;"></div>
                    <div class="time skeleton" style="width:60px;height:10px;"></div>
                </div>
            </div>
            <div class="post-content skeleton" style="height:60px;"></div>
        `
    fragment.appendChild(dummy_el)
  }
  container_el.appendChild(fragment)
}

///

//function view_timeline_update_reaction(model, ev) {
//	let el;
//	const o = event_parse_reaction(ev);
//	if (!o)
//		return;
//	const ev_id = o.e;
//	const root = model.elements[ev_id];
//	if (!root)
//		return;
//
//	// Update reaction groups
//	el = find_node(`.reactions`, root);
//	el.innerHTML = render_reactions_inner(model, model.all_events[ev_id]);
//
//	// Update like button
//	if (ev.pubkey == model.pubkey) {
//		const reaction = model_get_reacts_to(model, model.pubkey, ev_id, R_SHAKA);
//		const liked = !!reaction;
//		const img = find_node("button.icon.heart > img", root);
//		const btn = find_node("button.icon.heart", root)
//		btn.classList.toggle("liked", liked);
//		btn.title = liked ? "Unlike" : "Like";
//		btn.disabled = false;
//		btn.dataset.liked = liked ? "yes" : "no";
//		btn.dataset.reactionId = liked ? reaction.id : "";
//		img.classList.toggle("dark-noinvert", liked);
//		img.src = liked ? IMG_EVENT_LIKED : IMG_EVENT_LIKE;
//	}
//}

function view_timeline_update_reaction(model, ev) {
  let el
  const o = event_parse_reaction(ev)
  if (!o) return
  const ev_id = o.e
  const root = model.elements[ev_id]
  if (!root) return

  // Update reaction groups
  el = find_node(`.reactions`, root)
  el.innerHTML = render_reactions_inner(model, model.all_events[ev_id])

  // Update like button
  if (ev.pubkey == model.pubkey) {
    const reaction = model_get_reacts_to(model, model.pubkey, ev_id, R_SHAKA)
    const liked = !!reaction
    const img = find_node("button.icon.heart > img", root)
    const btn = find_node("button.icon.heart", root)
    btn.classList.toggle("liked", liked)
    btn.title = liked ? "Unlike" : "Like"
    btn.disabled = false
    btn.dataset.liked = liked ? "yes" : "no"
    btn.dataset.reactionId = liked ? reaction.id : ""
    img.classList.toggle("dark-noinvert", liked)
    img.src = liked ? IMG_EVENT_LIKED : IMG_EVENT_LIKE
  }
}

function init_search() {
  const search_input = find_node("#main-search")
  search_input.addEventListener("keyup", (ev) => {
    if (ev.key === "Enter") {
      const query = search_input.value.toLowerCase()
      if (query) {
        GNOSTR.search_results = []
        for (const key in GNOSTR.all_events) {
          const event = GNOSTR.all_events[key]
          let found = false

          // Helper function to recursively search an object/array
          function searchInObject(obj) {
            if (typeof obj === "string") {
              return obj.toLowerCase().includes(query)
            }
            if (typeof obj === "number" || typeof obj === "boolean") {
              return String(obj).includes(query)
            }
            if (Array.isArray(obj)) {
              for (const item of obj) {
                if (searchInObject(item)) {
                  return true
                }
              }
            } else if (typeof obj === "object" && obj !== null) {
              for (const prop in obj) {
                if (Object.prototype.hasOwnProperty.call(obj, prop)) {
                  if (searchInObject(obj[prop])) {
                    return true
                  }
                }
              }
            }
            return false
          }

          if (searchInObject(event)) {
            found = true
          }

          if (found) {
            GNOSTR.search_results.push(event)
          }
        }
        switch_view(VM_SEARCH, { query: search_input.value })
      }
    }
  })
}

function view_mode_contains_event(model, ev, mode, opts = {}) {
  console.log(`view_mode_contains_event called. Mode: ${mode}, Event Kind: ${ev.kind}`)
  if (mode != VM_DM_THREAD && ev.kind == KIND_DM) {
    return false
  }
  switch (mode) {
    case VM_USER:
      return opts.pubkey && ev.pubkey == opts.pubkey
    case VM_FRIENDS:
      if (opts.hide_replys && event_is_reply(ev)) return false
      return ev.pubkey == model.pubkey || contact_is_friend(model.contacts, ev.pubkey)
    case VM_THREAD:
      if (ev.kind == KIND_SHARE) return false
      return ev.id == opts.thread_id || event_refs_event(ev, { id: opts.thread_id })
    case VM_NOTIFICATIONS:
      return event_tags_pubkey(ev, model.pubkey)
    case VM_DM_THREAD:
      if (ev.kind != KIND_DM) return false
      return (
        (ev.pubkey == opts.pubkey && event_tags_pubkey(ev, model.pubkey)) ||
        (ev.pubkey == model.pubkey && event_tags_pubkey(ev, opts.pubkey))
      )
    case VM_GNOSTR:
      const global_nip34_kinds = new Set([
        KIND_REPO_ANNOUNCE,
        KIND_REPO_STATE_ANNOUNCE,
        KIND_REPO_PATCH,
        KIND_REPO_PULL_REQ,
        KIND_REPO_PULL_REQ_UPDATE,
        KIND_REPO_ISSUE,
        KIND_REPO_STATUS_OPEN,
        KIND_REPO_STATUS_APPLIED,
        KIND_REPO_STATUS_CLOSED,
        KIND_REPO_STATUS_DRAFT,
      ])
      return global_nip34_kinds.has(ev.kind)
    case VM_NIP34:
      const nip34_kinds = new Set([
        KIND_REPO_ANNOUNCE,
        KIND_REPO_STATE_ANNOUNCE,
        KIND_REPO_PATCH,
        KIND_REPO_PULL_REQ,
        KIND_REPO_PULL_REQ_UPDATE,
        KIND_REPO_ISSUE,
        KIND_REPO_STATUS_OPEN,
        KIND_REPO_STATUS_APPLIED,
        KIND_REPO_STATUS_CLOSED,
        KIND_REPO_STATUS_DRAFT,
      ])
      return nip34_kinds.has(ev.kind) && model.contacts.friends.has(ev.pubkey)
    case VM_NIP34_DETAIL:
      console.log(
        `view_mode_contains_event: Filtering for VM_NIP34_DETAIL. Event ID: ${ev.id}, Repo ID from opts: ${opts.repo_id}`,
      )
      const all_nip34_kinds = new Set([
        KIND_REPO_ANNOUNCE,
        KIND_REPO_STATE_ANNOUNCE,
        KIND_REPO_PATCH,
        KIND_REPO_PULL_REQ,
        KIND_REPO_PULL_REQ_UPDATE,
        KIND_REPO_ISSUE,
        KIND_REPO_STATUS_OPEN,
        KIND_REPO_STATUS_APPLIED,
        KIND_REPO_STATUS_CLOSED,
        KIND_REPO_STATUS_DRAFT,
      ])
      if (!all_nip34_kinds.has(ev.kind)) {
        console.log(`view_mode_contains_event: Event ${ev.id} is not a NIP-34 kind.`)
        return false
      }
      // Check for the 'a' tag matching the repo_id
      for (const tag of ev.tags) {
        if (tag[0] === "a" && tag[1] === opts.repo_id) {
          console.log(`view_mode_contains_event: Event ${ev.id} matches repo ID ${opts.repo_id}.`)
          return true
        }
      }
      console.log(`view_mode_contains_event: Event ${ev.id} does NOT match repo ID ${opts.repo_id}.`)
      return false
    case VM_SEARCH:
      return model.search_results.includes(ev)
  }
  return false
}

function event_is_renderable(ev = {}) {
  return (
    ev.kind == KIND_NOTE ||
    ev.kind == KIND_SHARE ||
    ev.kind == KIND_DM ||
    ev.kind == KIND_REPO_ANNOUNCE ||
    ev.kind == KIND_REPO_STATE_ANNOUNCE ||
    ev.kind == KIND_REPO_PATCH ||
    ev.kind == KIND_REPO_PULL_REQ ||
    ev.kind == KIND_REPO_PULL_REQ_UPDATE ||
    ev.kind == KIND_REPO_ISSUE ||
    ev.kind == KIND_REPO_STATUS_OPEN ||
    ev.kind == KIND_REPO_STATUS_APPLIED ||
    ev.kind == KIND_REPO_STATUS_CLOSED ||
    ev.kind == KIND_REPO_STATUS_DRAFT
  )
}

function get_default_max_depth(gnostr, view) {
  return view.max_depth || gnostr.max_depth
}

function get_thread_max_depth(gnostr, view, root_id) {
  if (!view.depths[root_id]) return get_default_max_depth(gnostr, view)
  return view.depths[root_id]
}

function get_thread_root_id(gnostr, id) {
  const ev = gnostr.all_events[id]
  if (!ev) {
    log_debug("expand_thread: no event found?", id)
    return null
  }
  return ev.refs && ev.refs.root
}

function switch_view(mode, opts) {
  view_timeline_apply_mode(GNOSTR, mode, opts)
}

function toggle_hide_replys(el) {
  const hide = el.innerText == "Hide Replys"
  switch_view(VM_FRIENDS, { hide_replys: hide })
  el.innerText = hide ? "Show Replys" : "Hide Replys"
}

function reset_notifications(model) {
  model.notifications.count = 0
  model.notifications.last_viewed = new_creation_time()
  update_notifications(model)
}

function html2el(html) {
  const div = document.createElement("div")
  div.innerHTML = html
  return div.firstChild
}

function init_timeline(model) {
  const el = view_get_timeline_el()
  el.addEventListener("click", onclick_timeline)
}
function onclick_timeline(ev) {
  if (ev.target.matches(".username[data-pubkey]")) {
    open_profile(ev.target.dataset.pubkey)
  }
}

function init_my_pfp(model) {
  find_nodes(`img[role='my-pfp']`).forEach((el) => {
    el.dataset.pubkey = model.pubkey
    el.addEventListener("error", onerror_pfp)
    el.addEventListener("click", onclick_pfp)
    el.classList.add("clickable")
  })
  find_nodes(`img[role='their-pfp']`).forEach((el) => {
    el.addEventListener("error", onerror_pfp)
    el.addEventListener("click", onclick_pfp)
    el.classList.add("clickable")
  })
}

function init_postbox(model) {
  find_node("#reply-content").addEventListener("input", oninput_post)
  find_node("#dm-post textarea").addEventListener("input", oninput_post)
  find_node("button[name='reply']").addEventListener("click", onclick_reply)
  find_node("button[name='reply-all']").addEventListener("click", onclick_reply)
  find_node("button[name='send']").addEventListener("click", onclick_send)
  find_node("button[name='send-dm']").addEventListener("click", onclick_send_dm)
}
async function onclick_reply(ev) {
  do_send_reply(ev.target.dataset.all == "1")
}
async function onclick_send(ev) {
  const el = find_node("#reply-modal")
  const pubkey = await get_pubkey()
  const el_input = el.querySelector("#reply-content")
  let post = {
    pubkey,
    kind: KIND_NOTE,
    created_at: new_creation_time(),
    content: el_input.value,
    tags: [],
  }
  post.id = await nostrjs.calculate_id(post)
  post = await sign_event(post)
  broadcast_event(post)

  // Reset UI
  el_input.value = ""
  trigger_postbox_assess(el_input)
  close_modal(el)
}
async function onclick_send_dm(ev) {
  const pubkey = await get_pubkey()
  const el = find_node("#dm-post")
  const el_input = el.querySelector("textarea")
  const target = view_get_timeline_el().dataset.pubkey
  let post = {
    pubkey,
    kind: KIND_DM,
    created_at: new_creation_time(),
    content: await window.nostr.nip04.encrypt(target, el_input.value),
    tags: [["p", target]],
  }
  post.id = await nostrjs.calculate_id(post)
  post = await sign_event(post)
  broadcast_event(post)

  el_input.value = ""
  trigger_postbox_assess(el_input)
}
/* oninput_post checks the content of the textarea and updates the size
 * of it's element. Additionally I will toggle the enabled state of the sending
 * button.
 */
function oninput_post(ev) {
  trigger_postbox_assess(ev.target)
}
function trigger_postbox_assess(el) {
  el.style.height = `0px`
  el.style.height = `${el.scrollHeight}px`
  let btn = el.parentElement.querySelector("button[role=send]")
  if (btn) btn.disabled = el.value === ""
}
/* toggle_cw changes the active stage of the Content Warning for a post. It is
 * relative to the element that is pressed.
 */
function onclick_toggle_cw(ev) {
  const el = ev.target
  el.classList.toggle("active")
  const isOn = el.classList.contains("active")
  const input = el.parentElement.querySelector("input.cw")
  input.classList.toggle("hide", !isOn)
}

function onclick_any(ev) {
  console.log(
    "onclick_any triggered. Event target:",
    ev.target,
    "Action attribute on target:",
    ev.target.getAttribute("action"),
  )
  let el = ev.target
  // Check if we have a selection and don't bother with anything
  let selection = document.getSelection()
  if (selection && selection.isCollapsed == false && view_get_timeline_el().contains(selection.anchorNode)) {
    return
  }
  let action = el.getAttribute("action")
  if (action == null && el.tagName != "A") {
    const parent = find_parent(el, "[action]")
    if (parent) {
      const parent_action = parent.getAttribute("action")
      // This is a quick hijack for propogating clicks; further extending
      // this should be obvious.
      if (parent_action == "open-thread") {
        el = parent
        action = parent_action
      }
      // Also handle NIP-34 detail view propagation if the parent has the action
      if (parent_action == "open-nip34-detail") {
        el = parent
        action = parent_action
      }
    }
  }
  switch (action) {
    case "sign-in":
      signin()
      break
    case "open-view":
      switch_view(el.dataset.view)
      break
    case "close-media":
      close_media_preview()
      break
    case "close-modal":
      close_modal(el)
      break
    case "open-profile":
      open_profile(el.dataset.pubkey)
      break
    case "open-profile-editor":
      click_update_profile()
      break
    case "show-timeline-new":
      show_new()
      break
    case "show-timeline-more":
      view_timeline_show_more(GNOSTR)
      break
    case "open-thread":
      open_thread(el.dataset.threadId)
      break
    case "reply":
      send_reply(el.dataset.emoji, el.dataset.to)
      break
    case "delete":
      delete_post(el.dataset.evid)
      break
    case "reply-to":
      reply(el.dataset.evid)
      break
    case "react-like":
      click_toggle_like(el)
      break
    case "share":
      click_share(el)
      break
    case "open-thread":
      open_thread(el.dataset.threadId)
      break
    case "open-media":
      open_media_preview(el.src, el.dataset.type)
      break
    case "open-link":
      window.open(el.dataset.url, "_blank")
      break
    case "open-lud06":
      open_lud06(el.dataset.lud06)
      break
    case "show-event-json":
      on_click_show_event_details(el.dataset.evid)
      break
    case "open-event-options":
      on_click_show_event_details(el.dataset.evid)
      break
    case "confirm-delete":
      delete_post_confirm(el.dataset.evid)
      break
    case "mark-all-read":
      model_mark_dms_seen(GNOSTR)
      break
    case "toggle-hide-replys":
      toggle_hide_replys(el)
      break
    case "new-note":
      new_note()
      break
    case "open-nip34-detail":
      console.log(`onclick_any: Opening NIP-34 detail for repo_id: ${el.dataset.repoId}`)
      switch_view(VM_NIP34_DETAIL, { repo_id: el.dataset.repoId })
      break
    case "show-nip34-more":
      console.log("onclick_any: 'Show More' for NIP-34 clicked.")
      show_more_nip34_events(GNOSTR)
      break
  }
}
