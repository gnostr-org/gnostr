import { w2layout } from './w2layout.js'
import { w2sidebar } from './w2sidebar.js'
import { w2grid } from './w2grid.js'
import { query } from './query.js'
import { w2popup } from './w2popup.js'
import { GNOSTR, KIND_REPO_ANNOUNCE, KIND_REPO_STATE_ANNOUNCE, KIND_REPO_PATCH, KIND_REPO_PULL_REQ, KIND_REPO_PULL_REQ_UPDATE, KIND_REPO_ISSUE, KIND_REPO_STATUS_OPEN, KIND_REPO_STATUS_APPLIED, KIND_REPO_STATUS_CLOSED, KIND_REPO_STATUS_DRAFT, log_info } from './main.js';
import { render_repo_event_summary } from './ui/render.js';
import { model_get_profile } from './model.js';
export function init_nostr_git_forge() {
    console.log("Initializing Nostr Git Forge");

    const model = GNOSTR;

    if (!model) {
        console.error("GNOSTR is not available. Nostr Git Forge might not function correctly.");
        return;
    }

    let layout, sidebar, repo_grid;

    let config = {
        layout: {
            name: 'layout',
            padding: 0,
            panels: [
                { type: 'left', size: 200, resizable: true, minSize: 120 },
                { type: 'main', minSize: 550, overflow: 'hidden' }
            ]
        },
        sidebar: {
            name: 'sidebar',
            nodes: [
                { id: 'repositories', text: 'Repositories', group: true, expanded: true, nodes: [
                    { id: 'all_repos', text: 'All Repositories', icon: 'fa fa-folder-o', selected: true },
                    { id: 'my_repos', text: 'My Repositories', icon: 'fa fa-user' }
                ]},
                { id: 'issues', text: 'Issues', icon: 'fa fa-bug' },
                { id: 'pull_requests', text: 'Pull Requests', icon: 'fa fa-code-fork' }
            ],
            onClick(event) {
                console.log('Sidebar item clicked:', event.target);
                switch (event.target) {
                    case 'all_repos':
                        loadAllRepositories();
                        break;
                    case 'my_repos':
                        loadMyRepositories();
                        break;
                    case 'issues':
                        loadIssues();
                        break;
                    case 'pull_requests':
                        loadPullRequests();
                        break;
                    default:
                        layout.html('main', '<div style="padding: 10px;">Content for ' + event.target + ' will go here.</div>');
                        break;
                }
            }
        },
        repo_grid: {
            name: 'repo_grid',
            show: {
                toolbar: true,
                footer: true,
                header: true,
                lineNumbers: true
            },
                            columns: [
                                { field: 'name', text: 'Repository Name', size: '30%', sortable: true },
                                { field: 'description', text: 'Description', size: '50%', sortable: true },
                                { field: 'maintainers', text: 'Maintainers', size: '20%', sortable: true }
                            ],
                            records: [],
                            onSelect: function(event) {
                                const selected_repo_id = event.detail.recid;
                                const selected_repo = repo_grid.get(selected_repo_id);
                                if (selected_repo && selected_repo.event) {
                                    const repo_event = selected_repo.event;
                                    const details_html = render_repo_event_summary(model, repo_event);
                                    layout.html('main', `<div style="padding: 10px;">${details_html}</div>`);
                                }
                            },
                            onDblClick: function(event) {
                                const selected_repo_id = event.detail.recid;
                                const selected_repo = repo_grid.get(selected_repo_id);
                                if (selected_repo && selected_repo.event) {
                                    const raw_event_json = JSON.stringify(selected_repo.event, null, 2);
                                    w2popup.open({
                                        title   : 'Raw NIP-34 Event',
                                        body    : `<div style="padding: 10px; white-space: pre-wrap; overflow-y: auto; max-height: 400px;">${raw_event_json}</div>`,
                                        buttons : '<button class="w2ui-btn" onclick="w2popup.close()">Close</button>',
                                        width   : 800,
                                        height  : 600
                                    });
                                }
                            }
                        }
                    };
                layout = new w2layout(config.layout);
    sidebar = new w2sidebar(config.sidebar);
    repo_grid = new w2grid(config.repo_grid);

    layout.render('#layout');
    layout.html('left', sidebar);
    layout.html('main', repo_grid); // Initially show an empty grid for repositories

    // Initial load of all repositories
    loadAllRepositories();

    // Helper functions (to be implemented)
    let active_all_repos_sub_id = null;

    function loadAllRepositories() {
        console.log("Loading all repositories...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        if (!model || !model.pool) {
            console.warn("model or model.pool not available.");
            return;
        }

        // Unsubscribe from previous 'all_repos' subscription if active
        if (active_all_repos_sub_id) {
            model.pool.unsubscribe(active_all_repos_sub_id);
            active_all_repos_sub_id = null;
        }

        const sub_id = `git-forge-all-repos-${Date.now()}`;
        active_all_repos_sub_id = sub_id;
        let current_repos_map = new Map(); // Map: event_id -> repo_data

        const filter = {
            kinds: [KIND_REPO_ANNOUNCE],
            // limit: 100 // Fetch a reasonable number of events
        };

        log_info(`Subscribing to all NIP-34 repo announce events with sub_id: ${sub_id}`);
        model.pool.subscribe(sub_id, [filter]);

        // Custom event handler for this specific subscription
        const handleNip34Event = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && ev.kind === KIND_REPO_ANNOUNCE) {
                if (current_repos_map.has(ev.id)) return; // Avoid processing duplicates

                let repo_name = "Unknown";
                let description = "";
                let maintainers = [];

                for (const tag of ev.tags) {
                    if (tag[0] === "d") {
                        repo_name = tag[1];
                    } else if (tag[0] === "description") {
                        description = tag[1];
                    } else if (tag[0] === "maintainers") {
                        maintainers = tag.slice(1).map(pk => model_get_profile(model, pk).data.name || pk.substring(0, 8));
                    }
                }
                const repo_data = { recid: ev.id, name: repo_name, description: description, maintainers: maintainers.join(", "), event: ev };
                current_repos_map.set(ev.id, repo_data);

                // Add to grid immediately for a more responsive feel
                repo_grid.add([repo_data]);
            }
        };

        const handleNip34Eose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. All repositories loaded.`);
                // All events for this subscription have been received.
                // The grid should already be populated by handleNip34Event.
                // We can potentially unsubscribe here, but keeping it open might be useful for live updates.
                // For now, let's unsubscribe to avoid resource leaks.
                model.pool.unsubscribe(sub_id);
                active_all_repos_sub_id = null; // Mark as no longer active
            }
        };

        // Attach custom handlers for this subscription's lifecycle
        // Temporarily modify model.pool's event handlers to catch our subscription's events
        // This is still a bit hacky. A more robust RelayPool implementation would allow per-subscription handlers.
        const originalOnEvent = model.pool.onfn.event;
        const originalOnEose = model.pool.onfn.eose;

        model.pool.onfn.event = (relay, received_sub_id, ev) => {
            handleNip34Event(relay, received_sub_id, ev);
            originalOnEvent?.(relay, received_sub_id, ev); // Call original handler
        };
        model.pool.onfn.eose = (relay, received_sub_id) => {
            handleNip34Eose(relay, received_sub_id);
            originalOnEose?.(relay, received_sub_id); // Call original handler
        };
    }

    let active_my_repos_sub_id = null;

    function loadMyRepositories() {
        console.log("Loading my repositories...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        if (!model || !model.pool || !model.pubkey) {
            console.warn("model, model.pool, or model.pubkey not available. Cannot load user repositories.");
            layout.html('main', '<div style="padding: 10px;">Please sign in to view your repositories.</div>');
            return;
        }

        // Unsubscribe from previous 'my_repos' subscription if active
        if (active_my_repos_sub_id) {
            model.pool.unsubscribe(active_my_repos_sub_id);
            active_my_repos_sub_id = null;
        }

        const sub_id = `git-forge-my-repos-${Date.now()}`;
        active_my_repos_sub_id = sub_id;
        let current_repos_map = new Map(); // Map: event_id -> repo_data

        const filter = {
            kinds: [KIND_REPO_ANNOUNCE],
            authors: [model.pubkey]
            // limit: 100 // Fetch a reasonable number of events
        };

        log_info(`Subscribing to current user's NIP-34 repo announce events with sub_id: ${sub_id}`);
        model.pool.subscribe(sub_id, [filter]);

        // Custom event handler for this specific subscription
        const handleNip34Event = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && ev.kind === KIND_REPO_ANNOUNCE) {
                if (current_repos_map.has(ev.id)) return; // Avoid processing duplicates

                let repo_name = "Unknown";
                let description = "";
                let maintainers = [];

                for (const tag of ev.tags) {
                    if (tag[0] === "d") {
                        repo_name = tag[1];
                    } else if (tag[0] === "description") {
                        description = tag[1];
                    } else if (tag[0] === "maintainers") {
                        maintainers = tag.slice(1).map(pk => model_get_profile(model, pk).data.name || pk.substring(0, 8));
                    }
                }
                const repo_data = { recid: ev.id, name: repo_name, description: description, maintainers: maintainers.join(", "), event: ev };
                current_repos_map.set(ev.id, repo_data);

                // Add to grid immediately for a more responsive feel
                repo_grid.add([repo_data]);
            }
        };

        const handleNip34Eose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. All user repositories loaded.`);
                model.pool.unsubscribe(sub_id);
                active_my_repos_sub_id = null; // Mark as no longer active
            }
        };

        // Attach custom handlers for this subscription's lifecycle
        const originalOnEvent = model.pool.onfn.event;
        const originalOnEose = model.pool.onfn.eose;

        model.pool.onfn.event = (relay, received_sub_id, ev) => {
            handleNip34Event(relay, received_sub_id, ev);
            originalOnEvent?.(relay, received_sub_id, ev); // Call original handler
        };
        model.pool.onfn.eose = (relay, received_sub_id) => {
            handleNip34Eose(relay, received_sub_id);
            originalOnEose?.(relay, received_sub_id); // Call original handler
        };
    }

    let active_issues_sub_id = null;

    function loadIssues() {
        console.log("Loading issues...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        // Update columns for issues
        repo_grid.columns = [
            { field: 'title', text: 'Issue Title', size: '50%', sortable: true },
            { field: 'status', text: 'Status', size: '20%', sortable: true },
            { field: 'repository', text: 'Repository', size: '30%', sortable: true }
        ];
        repo_grid.refreshColumns(); // Refresh columns display

        if (!model || !model.pool) {
            console.warn("model or model.pool not available. Cannot load issues.");
            return;
        }

        if (active_issues_sub_id) {
            model.pool.unsubscribe(active_issues_sub_id);
            active_issues_sub_id = null;
        }

        const sub_id = `git-forge-issues-${Date.now()}`;
        active_issues_sub_id = sub_id;
        let current_issues_map = new Map();

        const filter = {
            kinds: [KIND_REPO_ISSUE]
        };

        log_info(`Subscribing to NIP-34 issue events with sub_id: ${sub_id}`);
        model.pool.subscribe(sub_id, [filter]);

        const handleIssueEvent = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && ev.kind === KIND_REPO_ISSUE) {
                if (current_issues_map.has(ev.id)) return;

                let issue_title = "Untitled Issue";
                let status = "Unknown"; // Default status
                let repo_name = "Unknown Repository";

                for (const tag of ev.tags) {
                    if (tag[0] === "title") {
                        issue_title = tag[1];
                    } else if (tag[0] === "status") {
                        status = tag[1];
                    } else if (tag[0] === "d") { // Repository ID tag
                        repo_name = tag[1]; // Using 'd' tag value as repo name for now
                    }
                }
                const issue_data = { recid: ev.id, title: issue_title, status: status, repository: repo_name, event: ev };
                current_issues_map.set(ev.id, issue_data);
                repo_grid.add([issue_data]);
            }
        };

        const handleIssueEose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. All issues loaded.`);
                model.pool.unsubscribe(sub_id);
                active_issues_sub_id = null;
            }
        };

        const originalOnEvent = model.pool.onfn.event;
        const originalOnEose = model.pool.onfn.eose;

        model.pool.onfn.event = (relay, received_sub_id, ev) => {
            handleIssueEvent(relay, received_sub_id, ev);
            originalOnEvent?.(relay, received_sub_id, ev);
        };
        model.pool.onfn.eose = (relay, received_sub_id) => {
            handleIssueEose(relay, received_sub_id);
            originalOnEose?.(relay, received_sub_id);
        };
    }

    let active_pull_requests_sub_id = null;

    function loadPullRequests() {
        console.log("Loading pull requests...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        // Update columns for pull requests
        repo_grid.columns = [
            { field: 'title', text: 'Pull Request Title', size: '50%', sortable: true },
            { field: 'status', text: 'Status', size: '20%', sortable: true },
            { field: 'repository', text: 'Repository', size: '30%', sortable: true }
        ];
        repo_grid.refreshColumns(); // Refresh columns display

        if (!model || !model.pool) {
            console.warn("model or model.pool not available. Cannot load pull requests.");
            return;
        }

        if (active_pull_requests_sub_id) {
            model.pool.unsubscribe(active_pull_requests_sub_id);
            active_pull_requests_sub_id = null;
        }

        const sub_id = `git-forge-pull-requests-${Date.now()}`;
        active_pull_requests_sub_id = sub_id;
        let current_pull_requests_map = new Map();

        const filter = {
            kinds: [KIND_REPO_PULL_REQ, KIND_REPO_PULL_REQ_UPDATE]
        };

        log_info(`Subscribing to NIP-34 pull request events with sub_id: ${sub_id}`);
        model.pool.subscribe(sub_id, [filter]);

        const handlePullRequestEvent = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && (ev.kind === KIND_REPO_PULL_REQ || ev.kind === KIND_REPO_PULL_REQ_UPDATE)) {
                if (current_pull_requests_map.has(ev.id)) return;

                let pr_title = "Untitled Pull Request";
                let status = "Unknown"; // Default status
                let repo_name = "Unknown Repository";

                for (const tag of ev.tags) {
                    if (tag[0] === "title") {
                        pr_title = tag[1];
                    } else if (tag[0] === "status") {
                        status = tag[1];
                    } else if (tag[0] === "d") { // Repository ID tag
                        repo_name = tag[1]; // Using 'd' tag value as repo name for now
                    }
                }
                const pr_data = { recid: ev.id, title: pr_title, status: status, repository: repo_name, event: ev };
                current_pull_requests_map.set(ev.id, pr_data);
                repo_grid.add([pr_data]);
            }
        };

        const handlePullRequestEose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. All pull requests loaded.`);
                model.pool.unsubscribe(sub_id);
                active_pull_requests_sub_id = null;
            }
        };

        const originalOnEvent = model.pool.onfn.event;
        const originalOnEose = model.pool.onfn.eose;

        model.pool.onfn.event = (relay, received_sub_id, ev) => {
            handlePullRequestEvent(relay, received_sub_id, ev);
            originalOnEvent?.(relay, received_sub_id, ev);
        };
        model.pool.onfn.eose = (relay, received_sub_id) => {
            handlePullRequestEose(relay, received_sub_id);
            originalOnEose?.(relay, received_sub_id);
        };
    }
}
