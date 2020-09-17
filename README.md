# for-each-account

Do cross AWS account work more efficiently

## Install

Via GitHub Releases

```sh
$ curl --tlsv1.2 -L "https://github.com/softprops/for-each-account/releases/download/v0.1.0/for-each-account-$(uname -s)-$(uname -m).tar.gz" \
  | tar -xz -C ~/bin
```

## usage

AWS [Organizations](https://aws.amazon.com/organizations/) allow you consolidate multiple AWS accounts under the umbrella a one organizational unit.

Often you'll find you need to preform an operation across all subaccounts of that organization.
This tool helps you do that. It assumes only that you've defined an assumable role that enables that operation to be performed in each sub account

This following will iterate over all subaccounts assuming a role within those accounts and execute a given command as the role

```sh
AWS_PROFILE=your-root-organization-iam-user \
    for-each-account \
    --role role-name \
    --command 'command to run'
```


Doug Tangren (softprops) 2020