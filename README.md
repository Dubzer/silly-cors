## Silly CORS

![Header](https://github.com/Dubzer/silly-cors/assets/18244287/cc653a45-dfd4-4bfe-b5bc-ea1393e91e40)

Hate browsers for CORS errors? Don't care about security and just want your thing to work? Thanks to this project, you can go silly with CORS!

Silly CORS is a cors-proxy that sits between the browser and the recipient of the request. It automatically adds the necessary headers to satisfy browsers' cross-origin requirements.


> Note that CORS was not designed to annoy you, but is actually quite useful when properly configured. You shouldn't use CORS proxies in production.

### Requirements

This project is designed to be used in Docker behind a reverse proxy. If you don't have one, take a look at [traefik](https://traefik.io), using which will allow you to easily get a minimal setup with SSL certificates.

Architecture: ARM64 or x86_64 (AMD64)

### Setup

Use following image with `docker run` or `docker-compose.yml`:
```
ghcr.io/dubzer/silly-cors:latest
```

Then add Silly CORS to your reverse-proxy config, forwarding to port 3001 (or custom from env variable).

### Configuration

Silly CORS uses **environment variables** for configuration. All of them are optional.

- `PORT` - by default is **3001**.
- `SECRET` - arbitrary string that can be used for auth. Empty by default (without auth).

### API

Everything related to the proxy starts with **Silly**.

#### Request headers
- `Silly-Host` - your desired destination host. It's **required**.
- `Silly-Secret` - if you've configured `SECRET` env variable, place it's value in this header.

#### Handling errors
It's important to properly handle any unexpected scenarios.

All errors from Silly CORS will have `Silly-Response=true` header in them. So you can check for it, and for example, log an error.

#### Usage example
Let's imagine we've started to get tired from all this sillines and want to reduce its amount by calling HTTP method **DELETE** on `https://api.service.example/cats/1`.


To do this, change the URL host to one where Silly CORS is deployed. In this example, it's `https://silly-cors.deployment`. Then, place a real host in the `Silly-Host` header. 

After awaiting a promise, we make sure that no errors has occured in Silly CORS by checking the header `Silly-Response`.

```js
const response = await fetch("https://silly-cors.deployment/cats/1", {
    method: "DELETE", 
    headers: {"Silly-Host": "api.service.example"}
});

if (result.headers.get("Silly-Response")) {
    console.error("Oops! Silliness overload", response, await response.text());
    return;
}

return result.json();
```