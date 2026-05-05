#!/usr/bin/osascript
tell application "Finder"
	set myFolder to choose folder
	set myFiles to files of myFolder whose name extension is in {"svg", "jpg", "jpeg", "tif", "tiff", "png", "JPG", "JPEG", "TIF", "TIFF", "PNG"}
	if (exists folder "app" of folder myFolder) is false then
		make folder at myFolder with properties {name:"app"}
	end if
	set widthChoices to {"1600", "1024", "800", "400"}
	set widthChoice to {choose from list widthChoices with prompt "Pick a width"}
	set qualityChoices to {"high", "normal", "low"}
	set qualityChoice to {choose from list qualityChoices with prompt "Pick a quality"}
	set formatChoices to {"jpeg", "png", "tiff"}
	set formatChoice to {choose from list formatChoices with prompt "Pick a format"}
	set imgPaths to ""
	repeat with myFile in myFiles
		set imgPaths to imgPaths & quoted form of POSIX path of (myFile as text) & " "
	end repeat
	set destinationFolder to myFolder & "app:" as string
	set destinationFolder to quoted form of POSIX path of destinationFolder
	do shell script "sips -s format " & formatChoice & " -s formatOptions " & qualityChoice & " -Z " & widthChoice & " -s dpiWidth 72 -s dpiHeight 72 -m '/System/Library/ColorSync/Profiles/sRGB Profile.icc' " & imgPaths & " --out " & destinationFolder
	if (exists folder "app@2x" of folder myFolder) is false then
		make folder at myFolder with properties {name:"app@2x"}
	end if
	set destinationFolder to myFolder & "app@2x:" as string
	set destinationFolder to quoted form of POSIX path of destinationFolder
	do shell script "sips -s format " & formatChoice & " -s formatOptions " & qualityChoice & " -Z " & widthChoice & " -s dpiWidth 144 -s dpiHeight 144 -m '/System/Library/ColorSync/Profiles/sRGB Profile.icc' " & imgPaths & " --out " & destinationFolder
	display dialog "Done !"
end tell
