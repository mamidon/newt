#!/usr/local/bin/pwsh

function Approve-Files {
	param(
		[string]
		$candidateSuffix,
		[string]
		$approvedSuffix
	)

	function Replace-Suffix {
		param (
			[string]
			$input,
			[string]
			$oldSuffix,
			[string]
			$newSuffix
		)

		"$($input.Replace($oldSuffix, ''))$newSuffix"
	}

	Get-ChildItem -Recurse . | ? {
		$_.Name.EndsWith($candidateSuffix) 
	} | % {
		touch $(Replace-Suffix($_.Name, $candidateSuffix, $approvedSuffix))
		$_
	} | ? {
		(Compare-Object (Get-Content $_.Name) (Get-Content (Replace-Suffix($_.Name, $candidateSuffix, $approvedSuffix)))).Count -gt 0
	} | % {
		diffmerge $_.Name (Replace-Suffix($_.Name, $candidateSuffix, $approvedSuffix))
	}
}

function Approve-Tokens {
	$inputSuffix = ".tokens.newt"
	$outputSuffix = ".tokens.approval"
	$approvedSuffix = ".token.approved"

	Get-ChildItem -Recurse . | ? {
		$_.Name.EndsWith($inputSuffix) 
	} | % {
		& cargo run -- --entry-file $_.Name --output-mode tokens
	}

	Approve-Files -candidateSuffix $outputSuffix -approvedSuffix $approvedSuffix
}
