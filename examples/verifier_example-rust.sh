#!/usr/bin/env ruby

BIN = ENV['BIN'] || 'pact'

puts "=> Starting API"
pipe = IO.popen("ruby examples/api.rb")
sleep 2

puts "=> Running Pact"
puts "=> Test FAILING Pact"
res = `#{BIN} verifier --hostname localhost --port 4567 --file ./examples/fail.json --state-change-url http://localhost:4567/provider-state`
puts res

puts "=> Test SUCCESSFUL Pact"
res = `#{BIN} verifier --hostname localhost --port 4567 --file ./examples/fail.json --file ./examples/another-they.json --state-change-url http://localhost:4567/provider-state`
puts res

puts "=> Shutting down API"
if RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
  system("taskkill /im #{pipe.pid}  /f /t >nul 2>&1")
else
  Process.kill 'TERM', pipe.pid
end

puts "Test exit status: #{res}"
puts
