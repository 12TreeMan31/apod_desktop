PREFIX = /usr

install: 
	cargo build --release
	sudo cp -f ./target/release/apod_desktop ${DESTDIR}${PREFIX}/bin
	sudo chmod 755 ${DESTDIR}${PREFIX}/bin/apod_desktop
	sudo cp apod.service /etc/systemd/user/
	sudo systemctl daemon-reload