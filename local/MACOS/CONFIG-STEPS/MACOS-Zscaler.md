Copy certificate to a permanent location:
	```bash
	mkdir -p "$HOME/certs"
	CERT_SRC=$(find "$HOME/Downloads/ZscalerRootCerts" -type f -name "*.crt" ! -path "*/__MACOSX/*" ! -name "._*" -print -quit)
	echo "Using certificate: $CERT_SRC"
	[ -n "$CERT_SRC" ] && cp -f "$CERT_SRC" "$HOME/certs/zscaler-root.crt"
	ls -lh "$HOME/certs/zscaler-root.crt"
	```

Import certificate into your login keychain (no admin):
	```bash
	security add-trusted-cert -k "$HOME/Library/Keychains/login.keychain-db" "$HOME/certs/zscaler-root.crt" \
	  || open "$HOME/certs/zscaler-root.crt"
	```
