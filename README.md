

<h1 align="center">
🚸
<br/>
  aws-crossing
</h1>

<p align="center">
   Do cross AWS account work more efficiently
</p>

<div align="center">
  <a alt="GitHub Actions" href="https://github.com/softprops/aws-crossing/actions">
    <img src="https://github.com/softprops/aws-crossing/workflows/Main/badge.svg"/>
  </a>
  <a alt="license" href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-brightgreen.svg"/>
  </a>
</div>

<br />

## what now?

AWS [Organizations](https://aws.amazon.com/organizations/) allow you consolidate multiple AWS accounts under the umbrella of one organizational unit.

Often you'll find you need to perform an operation across all subaccounts of that organization.
This tool helps you do that. It assumes only that you've defined an assumable IAM role that enables that operation to be performed in each sub account.

## install

Via Homebrew

```sh
$ brew install softprops/tools/aws-crossing
```

Via GitHub Releases

```sh
$ curl --tlsv1.2 -L "https://github.com/softprops/aws-crossing/releases/download/v0.1.1/aws-crossing-$(uname -s)-$(uname -m).tar.gz" \
  | tar -xz -C ~/bin
```

## usage

This following command will iterate over all subaccounts assuming a role within those accounts and execute a given command as the role

```sh
AWS_PROFILE=your-root-organization-iam-user \
    aws-crossing \
    --role role-name \
    --command 'command to run or script to call'
```

You can also inline a chain of commands by wrapping your accomand with `sh -c "..."`

```sh
AWS_PROFILE=your-root-organization-iam-user \
    aws-crossing \
    --role role-name \
    --command 'sh -c "echo $AWS_ACCOUNT_ID && aws s3 ls | wc -l"'
```

## how it works

This tool will use your current aws credentials to list all accounts. You'll need the `organizations:ListAccounts` account permission for this. 

The tool will then iterate over those accounts an create a temporary session within that account assume a provider role. 

You can then execute an arbitary command that will have those credentials made available in addition to an environment variable `AWS_ACCOUNT_ID` which is the current account the command is being executed for.


Doug Tangren (softprops) 2020