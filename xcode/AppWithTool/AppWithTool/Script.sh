#!/bin/sh
# Initialize global variables.
{
  declare SCRIPT_INVOKED_NAME="${BASH_SOURCE[${#BASH_SOURCE[@]}-1]}"
  export  SCRIPT_INVOKED_NAME
  declare SCRIPT_NAME="${SCRIPT_INVOKED_NAME##*/}"
  export  SCRIPT_NAME
  declare SCRIPT_INVOKED_PATH="$( dirname "${SCRIPT_INVOKED_NAME}" )"
  export  SCRIPT_INVOKED_PATH
  declare SCRIPT_PATH="$( cd "${SCRIPT_INVOKED_PATH}"; pwd )"
  export  SCRIPT_PATH
  declare SCRIPT_RUN_DATE="$( date +%s )"
  export  SCRIPT_RUN_DATE
  
if [ ! -n DEBUG ];then
  echo "\nSCRIPT_INVOKED_NAME="$SCRIPT_INVOKED_NAME
  echo "SCRIPT_NAME="$SCRIPT_NAME
  echo "SCRIPT_INVOKED_PATH="$SCRIPT_INVOKED_PATH
  echo "SCRIPT_PATH="$SCRIPT_PATH
  echo "SCRIPT_RUN_DATE="$SCRIPT_RUN_DATE
  echo "\n"
fi

if [ ! -n DEBUG ];then
    ls $SCRIPT_INVOKED_PATH;
    echo "\n"
    open $SCRIPT_INVOKED_PATH/SanJuanGoddess.jpg
fi

$SCRIPT_INVOKED_PATH/./ToolX

}

addpath(){

if [ ! -z DEBUG ];then
    export PATH=$PATH:/Volumes/git/AppWithTool/Build/Products/Debug/
else
    echo
fi

export PATH=$PATH:$(PWD)
export PATH=$PATH:$SCRIPT_INVOKED_PATH
echo $PATH
}

list_user_home(){

if [ ! -n DEBUG ];then
    ## ls Desktop
    echo "\nDocuments:"
    ls Documents
    echo "\nDownloads:"
    ls Downloads
    echo "\nLibrary:"
    ls Library
    echo "\nMovies:"
    ls Movies
    echo "\nMusic:"
    ls Music
    echo "\nPictures:"
    ls Pictures
    ## open Pictures/Photos\ Library.photoslibrary
fi

}

## ls $(PWD)

echo "DEBUG="$DEBUG

if [ ! -n DEBUG ];then
    list_user_home;
else
    echo
fi
