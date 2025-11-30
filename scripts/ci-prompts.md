1. git push
2. run "which gh"
3. use gh utility to determine the related action that was just triggered (ELAPSED column)
4. sleep 120
5. read the errors from the action
6. fix first error
7. cargo c -j8
8. cargo t -j8
9. git add
10. run "which gnostr"
11. gnostr legit --help
12. use gnostr legit -m "commit message" to compose the commit
13. repeat