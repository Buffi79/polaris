 cargo build --release
 #RUST_LOG=debug ./target/release/polaris --foreground
 docker build -t polaris:latest .
 cd polaris-web
 npm run build
 npm run dev
#./target/release/polaris --foreground
