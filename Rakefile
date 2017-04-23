require 'os'
require 'fileutils'

target = "ghwcli"
tp = "target/release/#{target}"

if OS.windows? then
	target = "#{target}.exe"
	tp = "target\\release\\#{target}"
end

task :default do
	sh "cargo build --release"
end

task :upx => [:default] do
	if File.exists?(target) then
		File.delete(target)
	end
	sh "upx -9 #{tp} -o #{target}"
end

task :cleanwrk do
	FileUtils.rm_rf("__git__")
end

task :clean do
	sh "cargo clean"
	if File.exists?(target) then
		File.delete(target)
	end
end

task :cleanlock do
	File.delete("Cargo.lock")
end

task :test do
	sh "#{tp} --help"
	puts
	sh "#{tp} --version"
	puts
	sh "#{tp} clone stpettersens/touch"
	puts 
	if OS.windows? then
		sh "type __git__\\README.md"
	else
		sh "cat __git__/README.md"
	end
end
