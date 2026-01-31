import { w2layout } from './w2layout.js'
import { w2sidebar } from './w2sidebar.js'
import { w2grid } from './w2grid.js'
import { query } from './query.js'
import { KIND_REPO_ANNOUNCE, KIND_REPO_STATE_ANNOUNCE, KIND_REPO_PATCH, KIND_REPO_PULL_REQ, KIND_REPO_PULL_REQ_UPDATE, KIND_REPO_ISSUE, KIND_REPO_STATUS_OPEN, KIND_REPO_STATUS_APPLIED, KIND_REPO_STATUS_CLOSED, KIND_REPO_STATUS_DRAFT, log_info } from './util.js'; // Assuming these are in util.js for now
import { render_repo_event_summary } from './ui/render.js';
import { model_get_profile } from './model.js'; // Assuming this is in model.js

// Placeholder for GNOSTR model (will be passed or imported properly later)
const GNOSTR_MODEL = window.GNOSTR; // Assuming GNOSTR is globally available from main.js

let layout, sidebar, repo_grid;

export function init_nostr_git_forge() {
    console.log("Initializing Nostr Git Forge");

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
                    // Render detailed summary using render_repo_event_summary
                    const details_html = render_repo_event_summary(GNOSTR_MODEL, repo_event);
                    layout.html('main', `<div style="padding: 10px;">${details_html}</div>`);
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
    function loadAllRepositories() {
        console.log("Loading all repositories...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        if (!GNOSTR_MODEL || !GNOSTR_MODEL.pool) {
            console.warn("GNOSTR_MODEL or GNOSTR_MODEL.pool not available.");
            return;
        }

        const sub_id = `git-forge-all-repos-${Date.now()}`;
        let current_repos = [];

        GNOSTR_MODEL.pool.subscribe(
            sub_id,
            [
                { kinds: [KIND_REPO_ANNOUNCE] }
            ]
        );

        // Temporarily override on_pool_event and on_pool_eose to handle this specific subscription
        // NOTE: This is a simplified approach. In a real app, you'd manage subscriptions more robustly.
        const originalOnEvent = GNOSTR_MODEL.pool.onfn.event;
        const originalOnEose = GNOSTR_MODEL.pool.onfn.eose;

        GNOSTR_MODEL.pool.onfn.event = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && ev.kind === KIND_REPO_ANNOUNCE) {
                let repo_name = "Unknown";
                let description = "";
                let maintainers = "";

                for (const tag of ev.tags) {
                    if (tag[0] === "d") {
                        repo_name = tag[1];
                    } else if (tag[0] === "description") {
                        description = tag[1];
                    } else if (tag[0] === "maintainers") {
                        maintainers = tag.slice(1).map(pk => model_get_profile(GNOSTR_MODEL, pk).data.name || pk.substring(0, 8)).join(", ");
                    }
                }
                current_repos.push({ recid: ev.id, name: repo_name, description: description, maintainers: maintainers, event: ev });
            }
            originalOnEvent?.(relay, received_sub_id, ev); // Call original handler
        };

        GNOSTR_MODEL.pool.onfn.eose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. Populating grid.`);
                repo_grid.records = current_repos;
                repo_grid.refresh();
                layout.html('main', repo_grid); // Ensure grid is displayed after data load
                GNOSTR_MODEL.pool.unsubscribe(sub_id); // Unsubscribe after receiving all data

                // Restore original handlers
                GNOSTR_MODEL.pool.onfn.event = originalOnEvent;
                GNOSTR_MODEL.pool.onfn.eose = originalOnEose;
            }
            originalOnEose?.(relay, received_sub_id); // Call original handler
        };
    }

    function loadMyRepositories() {
        console.log("Loading my repositories...");
        repo_grid.clear();
        layout.html('main', repo_grid);

        if (!GNOSTR_MODEL || !GNOSTR_MODEL.pool || !GNOSTR_MODEL.pubkey) {
            console.warn("GNOSTR_MODEL, GNOSTR_MODEL.pool, or GNOSTR_MODEL.pubkey not available.");
            layout.html('main', '<div style="padding: 10px;">Please sign in to view your repositories.</div>');
            return;
        }

        const sub_id = `git-forge-my-repos-${Date.now()}`;
        let current_repos = [];

        GNOSTR_MODEL.pool.subscribe(
            sub_id,
            [
                { kinds: [KIND_REPO_ANNOUNCE], authors: [GNOSTR_MODEL.pubkey] }
            ]
        );

        const originalOnEvent = GNOSTR_MODEL.pool.onfn.event;
        const originalOnEose = GNOSTR_MODEL.pool.onfn.eose;

        GNOSTR_MODEL.pool.onfn.event = (relay, received_sub_id, ev) => {
            if (received_sub_id === sub_id && ev.kind === KIND_REPO_ANNOUNCE) {
                let repo_name = "Unknown";
                let description = "";
                let maintainers = "";

                for (const tag of ev.tags) {
                    if (tag[0] === "d") {
                        repo_name = tag[1];
                    } else if (tag[0] === "description") {
                        description = tag[1];
                    } else if (tag[0] === "maintainers") {
                        maintainers = tag.slice(1).map(pk => model_get_profile(GNOSTR_MODEL, pk).data.name || pk.substring(0, 8)).join(", ");
                    }
                }
                current_repos.push({ recid: ev.id, name: repo_name, description: description, maintainers: maintainers, event: ev });
            }
            originalOnEvent?.(relay, received_sub_id, ev); // Call original handler
        };

        GNOSTR_MODEL.pool.onfn.eose = (relay, received_sub_id) => {
            if (received_sub_id === sub_id) {
                log_info(`EOSE for ${sub_id}. Populating grid.`);
                repo_grid.records = current_repos;
                repo_grid.refresh();
                layout.html('main', repo_grid); // Ensure grid is displayed after data load
                GNOSTR_MODEL.pool.unsubscribe(sub_id); // Unsubscribe after receiving all data

                // Restore original handlers
                GNOSTR_MODEL.pool.onfn.event = originalOnEvent;
                GNOSTR_MODEL.pool.onfn.eose = originalOnEose;
            }
            originalOnEose?.(relay, received_sub_id); // Call original handler
        };
    }

    function loadIssues() {
        console.log("Loading issues...");
        layout.html('main', '<div style="padding: 10px;">Issues content will go here. (NIP-34 Kind 30617)</div>');
    }

    function loadPullRequests() {
        console.log("Loading pull requests...");
        layout.html('main', '<div style="padding: 10px;">Pull Requests content will go here. (NIP-34 Kinds 30618, 30619)</div>');
    }
}
