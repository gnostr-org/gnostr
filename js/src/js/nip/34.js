const KIND_REPO_ANNOUNCE = 30617;
const KIND_REPO_STATE_ANNOUNCE = 30618;
const KIND_REPO_PATCH = 1617;
const KIND_REPO_PULL_REQ = 1618;
const KIND_REPO_PULL_REQ_UPDATE = 1619;
const KIND_REPO_ISSUE = 1620;
const KIND_REPO_STATUS_OPEN = 1630;
const KIND_REPO_STATUS_APPLIED = 1631;
const KIND_REPO_STATUS_CLOSED = 1632;
const KIND_REPO_STATUS_DRAFT = 1633;
const KIND_RELAY_LIST = 10002;

const NIP34_REPO_KINDS = [
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
];

const NIP_34_KINDS = [...NIP34_REPO_KINDS, KIND_RELAY_LIST];
const NIP_EXPLORER_ITEMS = [
	{
		nip: "34",
		title: "NIP-34",
		href: "/nip/34",
		description: "Git repositories, announcements, patches, issues, and status events.",
	},
	{
		nip: "65",
		title: "NIP-65",
		href: "/settings",
		description: "Relay lists and relay preferences.",
	},
];

function is_nip34_repo_kind(kind) {
	return NIP34_REPO_KINDS.includes(kind);
}

function render_nip_explorer() {
	return html`<section class="nip-explorer">
		<header>
			<h2>NIP explorer</h2>
			<p>Browse supported NIPs and jump straight into their views.</p>
		</header>
		<ul class="nip-explorer-list">
			${NIP_EXPLORER_ITEMS.map((item) => html`
				<li class="nip-explorer-item">
					<div class="nip-explorer-item-head">
						<a class="nip-explorer-link" href="${item.href}">${item.title}</a>
						<span class="nip-explorer-id">/${item.nip}</span>
					</div>
					<p>${item.description}</p>
				</li>
			`).join("")}
		</ul>
	</section>`;
}
