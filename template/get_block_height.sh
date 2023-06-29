declare -a BLOCKHEIGHT
function get_block_height(){

	BLOCKHEIGHT=$(curl https://blockchain.info/q/getblockcount 2>/dev/null)
	echo $BLOCKHEIGHT
	#return $BLOCKHEIGHT

}
get_block_height