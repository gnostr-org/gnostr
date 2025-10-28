/* This file contains utility functions related to UI manipulation. Some code
 * may be specific to areas of the UI and others are more utility based. As
 * this file grows specific UI area code should be migrated to its own file.
 */

/* init_message_textareas finds all message textareas and updates their initial
 * height based on their content (0). This is so there is no jaring affect when
 * the page loads.
 */
function init_message_textareas() {
	const els = document.querySelectorAll(".post-input");
	for (const el of els) {
		trigger_postbox_assess(el);
	}
}

// update_notification_markers will find all markers and hide or show them
// based on the passed in state of 'active'. This applies to the navigation 
// icons.
function update_notification_markers(active, type) {
	let els = document.querySelectorAll(`.new-notifications[role='${type}']`)
	for (const el of els) {
		el.classList.toggle("hide", !active)
	}
}

/* newlines_to_br takes a string and converts all newlines to HTML 'br' tags.
 */
function newlines_to_br(str="") {
	return str.split("\n").reduce((acc, part, index) => {
		return acc + part + "<br/>";
	}, "");
}

function show_new() {
	view_timeline_show_new(DAMUS);
}

function click_share(el) {
	share(el.dataset.evid);
}

function click_toggle_like(el) {
	// Disable the button to prevent multiple presses
	el.disabled = true;
	if (el.dataset.liked == "yes") {
		delete_post(el.dataset.reactionId);
		return;
	}
	send_reply(R_SHAKA, el.dataset.reactingTo);
}

/* open_media_preview presents a modal to display an image via "url".
 */
function open_media_preview(url, type) {
	const el = find_node("#media-preview");
	el.showModal();
	find_node("img", el).src = url;
	// TODO handle different medias such as audio and video
	// TODO add loading state & error checking
}

/* close_media_preview closes any present media modal.
 */
function close_media_preview() {
	find_node("#media-preview").close();
}

function delete_post_confirm(evid) {
	if (!confirm("Are you sure you want to delete this post?"))
		return;
	const reason = (prompt("Why you are deleting this? Leave empty to not specify. Type CANCEL to cancel.") || "").trim()
	if (reason.toLowerCase() === "cancel")
		return;
	delete_post(evid, reason)
}

async function do_send_reply(all=false) {
	const modal = document.querySelector("#reply-modal");
	const replying_to = modal.querySelector("#replying-to");
	const evid = replying_to.dataset.evid;
	const reply_content_el = document.querySelector("#reply-content");
	const content = reply_content_el.value;
	await send_reply(content, evid, all);
	reply_content_el.value = "";
	close_modal(modal);
}

function update_reply_box(state="new") {
	const isnew = state == "new";
	const modal = document.querySelector("#reply-modal");
	modal.querySelector("#replying-to").classList.toggle("hide", isnew);
	modal.querySelector("header label").textContent = isnew ? "New Note" : "Replying To";
	modal.querySelector(".post-tools.new").classList.toggle("hide", !isnew);
	modal.querySelector(".post-tools.reply").classList.toggle("hide", isnew);
}

function new_note() {
	const modal = document.querySelector("#reply-modal");
	const inputbox = modal.querySelector("#reply-content");
	update_reply_box("new");
	inputbox.placeholder = "What's up?";
	modal.showModal();
	inputbox.focus();
}

function reply(evid) {
	const ev = DAMUS.all_events[evid]
	const modal = document.querySelector("#reply-modal")
	const replybox = modal.querySelector("#reply-content")
	const replying_to = modal.querySelector("#replying-to")
	update_reply_box("reply");
	replying_to.dataset.evid = evid
	replying_to.classList.remove("hide");
	replying_to.innerHTML = render_event_nointeract(DAMUS, ev, {
		is_composing: true, 
		nobar: true
	});
	replybox.placeholder = "Reply...";
	modal.showModal();
	replybox.focus();
}

function update_favicon(path) {
	let link = document.querySelector("link[rel~='icon']");
	const head = document.getElementsByTagName('head')[0]

	if (!link) {
		link = document.createElement('link');
		link.rel = 'icon';
		head.appendChild(link);
	}

	link.href = path;
}

// update_notifications updates the document title & visual indicators based on if the
// number of notifications that are unseen by the user.
function update_notifications(model) {
	let dm_count = 0;
	for (const item of model.dms) {
		if (item[1].new_count)
			dm_count += item[1].new_count;
	}
	
	const { count } = model.notifications;
	const suffix = "Yo Sup";
	const total = count + dm_count;
	document.title = total ? `(${total}) ${suffix}` : suffix;
	// TODO I broke the favicons; I will fix with later.
	//update_favicon(has_notes ? "img/damus_notif.svg" : "img/damus.svg");
	update_notification_markers(count, "activity");
	update_notification_markers(dm_count, "dm");
	// slight hack :)
	find_node("#header-tools button[action='mark-all-read']")
		.disabled = dm_count == 0;
}

async function get_pubkey(use_prompt=true) {
	if (!(window.nostr && window.nostr.getPublicKey)) {
		console.error("window.nostr.getPublicKey is unsupported");
		return;
	}
	try {
		return await window.nostr.getPublicKey()
	} catch (err) {
		console.error(err);
		return;
	}
}

function open_thread(thread_id) {
	view_timeline_apply_mode(DAMUS, VM_THREAD, { thread_id });
}

function close_modal(el) {
	find_parent(el, "dialog").close();
}

function on_click_show_event_details(evid) {
	const model = DAMUS;
	const ev = model.all_events[evid];
	if (!ev)
		return;
	const el = find_node("#event-details");
	el.showModal();
	find_node("code", el).innerText = JSON.stringify(ev, null, "\t");
}

function onclick_pfp(ev) {
	open_profile(ev.target.dataset.pubkey);	
}

function onerror_pfp(ev) {
	ev.target.src = IMG_NO_USER;
}
