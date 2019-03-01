
Get-ChildItem -Recurse . | ? {
	$_.Name.EndsWith(".txt") 
} | % {
	touch "$($_.Name).approved"
	$_
} | ? {
	(Compare-Object (Get-Content $_.Name) (Get-Content "$($_.Name).approved")).Count -gt 0
} | % {
	diffmerge $_.Name "$($_.Name).approved"
}