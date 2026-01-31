import { w2layout } from './w2layout.js'
import { w2sidebar } from './w2sidebar.js'
import { w2grid } from './w2grid.js'
import { query } from './query.js'
// Import other gnostr_js modules as needed
// import { GNOSTR, model_get_profile, KIND_REPO_ANNOUNCE, /* ... other kinds */ } from './main.js'
// import { render_repo_event_summary } from './ui/render.js'

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
            records: []
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
        // Simulate fetching data
        setTimeout(() => {
            const dummy_repos = [
                { recid: 1, name: 'nostr-protocol', description: 'The Nostr protocol specification', maintainers: 'fiatjaf' },
                { recid: 2, name: 'gnostr-client-js', description: 'A Nostr web client in JavaScript', maintainers: 'randymcmillan' },
                { recid: 3, name: 'awesome-nostr', description: 'A curated list of Nostr resources', maintainers: 'Various' }
            ];
            repo_grid.records = dummy_repos;
            repo_grid.refresh();
            layout.html('main', repo_grid);
        }, 500);
    }

    function loadMyRepositories() {
        console.log("Loading my repositories...");
        layout.html('main', '<div style="padding: 10px;">My Repositories content.</div>');
    }

    function loadIssues() {
        console.log("Loading issues...");
        layout.html('main', '<div style="padding: 10px;">Issues content.</div>');
    }

    function loadPullRequests() {
        console.log("Loading pull requests...");
        layout.html('main', '<div style="padding: 10px;">Pull Requests content.</div>');
    }
}
