<br/>
<div align="center">
<h3 align="center">Growtopia Proxy</h3>
<p align="center">
A utility for Growtopia
</p>
</div>

### Generating certificates
```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/C=ID/ST=JKT/L=Home/O=WorldDomination/CN=www.growtopia1.com"
```