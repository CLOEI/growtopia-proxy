<br/>
<div align="center">
<h3 align="center">Growtopia Proxy</h3>
<p align="center">
A utility for Growtopia
</p>
</div>

### Usage
Add an entry to the Host file replacing `www.growtopia1.com` and `www.growtopia.com` to point to your desired IP address.

### Current issue
Currently, proxy able to pass the subserver switching, but after that the server peer would send a quit packet. Let me know if you have any idea about this. 

There's also a issue with updating item database, server peer would just disconnect after the client peer send item database data.

Discord: `.cendy`

### Generating certificates
```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/C=ID/ST=JKT/L=Home/O=WorldDomination/CN=www.growtopia1.com"
```

### DoH
Current implementation use Cloudflare. You can change it to your own DoH server at `resolver.rs`