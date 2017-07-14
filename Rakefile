require 'os'
require 'fileutils'

target = "ghwcli"
tp = "target/release/#{target}"
srcin = "presrc/main.rs"
srcout = "src/main.rs"
ppcondition = "USE_CURL_LIB" # or USE_CURL_EXT

if OS.windows? then
	target = "#{target}.exe"
	tp = "target\\release\\#{target}"
	srcin = "presrc\\main.rs"
	srcout = "src\\main.rs"
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

task :configure do
	sh "fm --file _Cargo.toml --condition #{ppcondition} --out Cargo.toml"
	sh "fm --file #{srcin} --condition #{ppcondition} --out #{srcout}"
end

task :cleanwrk do
	FileUtils.rm_rf("_git_");
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
		sh "type _git_\\stpettersens\\touch\\master\\README.md"
	else
		sh "cat _git_/stpettersens/touch/master/README.md"
	end
end
