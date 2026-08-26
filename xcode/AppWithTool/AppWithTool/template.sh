#!/bin/sh
DATE=$(date +%s)
PID=$$

echo $DATE/$PID
echo $(( $DATE + $PID ))
echo $(( $DATE ^ $PID ))
echo $(( $DATE / $PID ))
echo $(( $PID  / $DATE ))

echo $PWD
echo $0
echo "\$\$=$$"
test whoami && whoami
echo PATH=$PATH
test git && which git
test make && which make
test bash && which bash
