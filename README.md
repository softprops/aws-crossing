# for-each-account

Do cross AWS account work more efficiently


## usage

AWS [Organizations](https://aws.amazon.com/organizations/) allow you consolidate multiple AWS accounts under the umbrella a one organizational unit.

Often you'll find you need to preform an operation across all subaccounts of that organization.
This tool helps you do that. It assumes only that you've defined an assumable role that enables that operation to be performed in each sub account

This following will iterate over all subaccounts assuming a role within those accounts and execute a given command as the role

```sh
AWS_PROFILE=your-root-organization-iam-user \
    cargo run -- \
    --role role-name \
    --command 'command to run'
```

Doug Tangren (softprops) 2020