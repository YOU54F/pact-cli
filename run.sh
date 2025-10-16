#!/bin/sh
set -e

BIN=${BIN:-pact-cli}
${BIN} --help
${BIN} pact-broker --help
${BIN} pactflow --help
${BIN} completions --help
${BIN} pact-broker docker --help
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

${BIN} pact-broker ruby stop || true
${BIN} pact-broker ruby start -d
${BIN} pact-broker ruby info
${BIN} pact-broker list-latest-pact-versions
${BIN} pact-broker create-environment --name name_foo1
${BIN} pact-broker create-environment --name name_foo2 --display-name display_name_foo
${BIN} pact-broker create-environment --name name_foo3 --display-name display_name_foo --contact-name contact_name_foo
${BIN} pact-broker create-environment --name name_foo4 --display-name display_name_foo --contact-name contact_name_foo --contact-email-address contact.email.address@foo.bar
export ENV_UUID=$(${BIN} pact-broker create-environment --name name_foo5 --output=id)
${BIN} pact-broker describe-environment --uuid $ENV_UUID
${BIN} pact-broker update-environment --uuid $ENV_UUID --name name_foo6
${BIN} pact-broker update-environment --uuid $ENV_UUID --name name_foo7 --display-name display_name_foo6
${BIN} pact-broker update-environment --uuid $ENV_UUID --name name_foo8 --contact-name contact_name_foo8
${BIN} pact-broker update-environment --uuid $ENV_UUID --name name_foo9 --contact-name contact_name_foo9 --contact-email-address contact_name_foo7
${BIN} pact-broker delete-environment --uuid $ENV_UUID
${BIN} pact-broker list-environments | awk -F '│' '{print $2}' | sed -n '3,$p' | sed '$d' | awk '{print $1}' | xargs -I {} ${BIN} pact-broker delete-environment --uuid {} 
${BIN} pact-broker create-environment --name production --production
${BIN} pact-broker publish tests/pacts -r
${BIN} pact-broker publish tests/pacts -a foo --branch bar
${BIN} pact-broker can-i-deploy --pacticipant GettingStartedOrderWeb --version foo --to prod || echo "can-i-deploy fails due to no verification result - expected"
${BIN} pact-broker can-i-deploy --pacticipant GettingStartedOrderWeb --version foo --to prod --dry-run
${BIN} pact-broker record-deployment --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} pact-broker record-undeployment --environment production --pacticipant GettingStartedOrderWeb
${BIN} pact-broker record-release --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} pact-broker record-support-ended --version foo --environment production --pacticipant GettingStartedOrderWeb
${BIN} pact-broker create-or-update-pacticipant --name foo --main-branch main --repository-url http://foo.bar
${BIN} pact-broker describe-pacticipant --name foo
${BIN} pact-broker list-pacticipants
${BIN} pact-broker create-webhook https://localhost --request POST --contract-published
export WEBHOOK_UUID=$(${BIN} pact-broker create-webhook https://localhost --request POST --contract-published | jq .uuid -r)
${BIN} pact-broker create-or-update-webhook https://foo.bar --request POST --uuid $WEBHOOK_UUID --provider-verification-succeeded
${BIN} pact-broker test-webhook --uuid $WEBHOOK_UUID
${BIN} pact-broker create-or-update-version --version foo --pacticipant foo --branch bar --tag baz
${BIN} pact-broker create-version-tag --version foo --pacticipant foo --tag bar
${BIN} pact-broker describe-version --pacticipant foo
${BIN} pact-broker can-i-merge --pacticipant foo --version foo
${BIN} pact-broker delete-branch --branch bar --pacticipant foo
${BIN} pact-broker describe-pacticipant --name foo
${BIN} pact-broker generate-uuid


${BIN} pact-broker ruby stop

