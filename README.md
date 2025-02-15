<br/>
<div align="center">
<h3 align="center">Growtopia Proxy</h3>
<p align="center">
A utility for Growtopia
</p>
</div>

### Usage
Add an entry to the Host file replacing `www.growtopia1.com` and `www.growtopia.com` to point to your desired IP address.

Then run it by using `cargo run --release`.
### Current issue
Currently, proxy able to pass the subserver switching, but after that the server peer would just disconnect later on. Let me know if you have any idea about this. 

There's also a issue with updating item database, server peer would just disconnect after the client peer send item database data.

Both issue already solved by changing type2 value to 0 in the http data and disabling `use_packet_server`. It would be better if we can find out what type2 is and what it does.

Discord: `.cendy`

### Generating certificates
```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/C=ID/ST=JKT/L=Home/O=WorldDomination/CN=www.growtopia1.com"
```

### DoH
Current implementation use Cloudflare. You can change it to your own DoH server at `resolver.rs`