#!/usr/local/bin/pwsh

function Replace-Suffix {
	param (
		[string]
		$text,
		[string]
		$oldSuffix,
		[string]
		$newSuffix
	)

	"$($text.Replace($oldSuffix, """"))$newSuffix"
}

function Approve-Files {
	param(
		[string]
		$candidateSuffix,
		[string]
		$approvedSuffix
	)

	$passed = 0
	$differences = 0

	Get-ChildItem -Recurse . | ? {
		$_.Name.EndsWith($candidateSuffix) 
	} | % {
		$approvedFilePathName = $(Replace-Suffix -text ($_.FullName) -oldSuffix $candidateSuffix -newSuffix $approvedSuffix)
		touch $approvedFilePathName
	
		$_
	} | ? {
		$approvedFilePathName = $(Replace-Suffix -text ($_.FullName) -oldSuffix $candidateSuffix -newSuffix $approvedSuffix)
		$candidateContent = Get-Content $_.FullName
		$approvedContent = Get-Content $approvedFilePathName

		if ($null -eq $approvedContent) {
			$approvedContent = ""
		}

		$delta = (Compare-Object -Ref $candidateContent $approvedContent -PassThru).Count
		
		if ($delta -gt 0) {
			$differences += 1
		} else {
			$passed += 1
		}

		$delta -gt 0
	} | % {
		$approvedFilePathName = $(Replace-Suffix -text ($_.FullName) -oldSuffix $candidateSuffix -newSuffix $approvedSuffix)
		diffmerge $_.FullName $approvedFilePathName
	}

	$message = "Completed approvals -- $passed/$($differences+$passed) passed"
	if ($differences -eq 0) {
		Write-Host -ForegroundColor Green $message
	} else {
		Write-Host -ForegroundColor Red $message
	}
}

function Approve-Tokens {
	$inputSuffix = ".tokens.newt"
	$outputSuffix = ".tokens.candidate"
	$approvedSuffix = ".tokens.approved"

	Get-ChildItem -Recurse -Filter "*.tokens.newt" . | % {
		$outputFile = Replace-Suffix -text ($_.FullName) -oldSuffix $inputSuffix -newSuffix $outputSuffix

		& cargo run --quiet -- --entry-file $_.FullName --output-mode tokens > $outputFile
	}

	Approve-Files -candidateSuffix $outputSuffix -approvedSuffix $approvedSuffix
}
