#!/bin/sh
set -e

BIN=${BIN:-pact}
${BIN} --help
${BIN} broker --help
${BIN} pactflow --help
${BIN} completions --help
${BIN} broker docker --help
${BIN} plugin --help
${BIN} plugin list --help
${BIN} plugin list installed --help
${BIN} plugin list known --help
${BIN} plugin env --help
${BIN} plugin install --help
${BIN} plugin remove --help
${BIN} plugin enable --help
${BIN} plugin disable --help
${BIN} plugin repository --help
${BIN} plugin repository validate --help
${BIN} plugin repository new --help
${BIN} plugin repository add-plugin-version --help
${BIN} plugin repository add-plugin-version git-hub --help
${BIN} plugin repository add-plugin-version file --help
${BIN} plugin repository add-all-plugin-versions --help
${BIN} plugin repository yank-version --help
${BIN} plugin repository list --help
${BIN} plugin repository list-versions --help
${BIN} stub --help
${BIN} verifier --help
${BIN} mock --help
${BIN} mock start --help
${BIN} mock list --help
${BIN} mock create --help
${BIN} mock verify --help
${BIN} mock shutdown --help
${BIN} mock shutdown-master --help

${BIN} broker ruby stop || true
${BIN} broker ruby start -d
${BIN} broker ruby info
${BIN} broker list-latest-pact-versions
${BIN} broker create-environment --name name_foo1
${BIN} broker create-environment --name name_foo2 --display-name display_name_foo
${BIN} broker create-environment --name name_foo3 --display-name display_name_foo --contact-name contact_name_foo
${BIN} broker create-environment --name name_foo4 --display-name display_name_foo --contact-name contact_name_foo --contact-email-address contact.email.address@foo.bar
export ENV_UUID=$(${BIN} broker create-environment --name name_foo5 --output=id)
${BIN} broker describe-environment --uuid $ENV_UUID
${BIN} broker update-environment --uuid $ENV_UUID --name name_foo6
${BIN} broker update-environment --uuid $ENV_UUID --name name_foo7 --display-name display_name_foo6
${BIN} broker update-environment --uuid $ENV_UUID --name name_foo8 --contact-name contact_name_foo8
${BIN} broker update-environment --uuid $ENV_UUID --name name_foo9 --contact-name contact_name_foo9 --contact-email-address contact_name_foo7
${BIN} broker delete-environment --uuid $ENV_UUID
${BIN} broker list-environments | awk -F '│' '{print $2}' | sed -n '3,$p' | sed '$d' | awk '{print $1}' | xargs -I {} ${BIN} broker delete-environment --uuid {} 
${BIN} broker create-environment --name production --production
${BIN} broker publish tests/pacts -r
${BIN} broker publish tests/pacts -a foo --branch bar
${BIN} broker can-i-deploy --pacticipant GettingStartedOrderWeb --version foo --to prod || echo "can-i-deploy fails due to no verification result - expected"
${BIN} broker can-i-deploy --pacticipant GettingStartedOrderWeb --version foo --to prod --dry-run
${BIN} broker record-deployment --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} broker record-undeployment --environment production --pacticipant GettingStartedOrderWeb
${BIN} broker record-release --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} broker record-support-ended --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} broker create-or-update-pacticipant --name foo --main-branch main --repository-url http://foo.bar
${BIN} broker describe-pacticipant --name foo
${BIN} broker list-pacticipants
${BIN} broker create-webhook https://localhost --request POST --contract-published
export WEBHOOK_UUID=$(${BIN} broker create-webhook https://localhost --request POST --contract-published | jq .uuid -r)
${BIN} broker create-or-update-webhook https://foo.bar --request POST --uuid $WEBHOOK_UUID --provider-verification-succeeded
${BIN} broker test-webhook --uuid $WEBHOOK_UUID
${BIN} broker create-or-update-version --version foo --pacticipant foo --branch bar --tag baz
${BIN} broker create-version-tag --version foo --pacticipant foo --tag bar
${BIN} broker describe-version --pacticipant foo
${BIN} broker can-i-merge --pacticipant foo --version foo
${BIN} broker delete-branch --branch bar --pacticipant foo
${BIN} broker describe-pacticipant --name foo
${BIN} broker generate-uuid


${BIN} broker ruby stop

