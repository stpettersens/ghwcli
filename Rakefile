require 'os'

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

task :clean do
	sh "cargo clean"
end

task :cleanlock do
	File.delete("Cargo.lock")
end

task :test do
	sh "#{tp}"
end
