act-automate:## 	run act -vr
	@export $(cat ~/GH_TOKEN.txt) && act -vbr -W .github/workflows/automate.yml && popd
act-static:## 	run act -vr
	@export $(cat ~/GH_TOKEN.txt) && act -vbr -W .github/workflows/static.yml && popd