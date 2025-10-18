#!/usr/bin/env ruby

BIN = ENV['BIN'] || 'pact-provider-verifier'

puts "=> Starting API"
pipe = IO.popen("ruby examples/api.rb")
sleep 2

puts "=> Running Pact"
puts "=> Test FAILING Pact"
res = `#{BIN} --provider-base-url http://localhost:4567 --pact-urls ./examples/fail.json --provider_states_setup_url http://localhost:4567/provider-state --provider_states_url http://localhost:4567/provider-states`
puts res

puts "=> Test SUCCESSFUL Pact"
res = `#{BIN} --provider-base-url http://localhost:4567 --pact-urls ./examples/me-they.json,./examples/another-they.json --provider_states_setup_url http://localhost:4567/provider-state --provider_states_url http://localhost:4567/provider-states`
puts res

puts "=> Shutting down API"
if RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
  system("taskkill /im #{pipe.pid}  /f /t >nul 2>&1")
else
  Process.kill 'TERM', pipe.pid
end

puts "Test exit status: #{res}"
puts
