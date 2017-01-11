task :default do
	sh "cargo build --release"
end

task :clean do
	sh "cargo clean"
end

task :cleanlock do
	File.delete("Cargo.lock")
end

