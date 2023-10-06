## Silly CORS

![Header](https://github.com/Dubzer/silly-cors/assets/18244287/cc653a45-dfd4-4bfe-b5bc-ea1393e91e40)

Hate CORS errors? Don't care about security and just want your thing to work? Thanks to this project, you can go silly with CORS!

Silly CORS is a cors-proxy that sits between the browser and request destination. It automatically adds the necessary headers to satisfy browsers' cross-origin requirements.


> Note that CORS was not designed to annoy you, but is actually quite useful when properly configured. You shouldn't use CORS proxies in production.

### Requirements

This service is designed to be used in Docker behind a reverse proxy. If you don't have one, take a look at [traefik](https://traefik.io), using which will allow you to easily get a minimal setup with SSL certificates.

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
- `Silly-Secret` - if you've configured `SECRET` env variable, place it's value in this header.

#### Handling errors
It's important to properly handle any unexpected scenarios.

All errors from Silly CORS will have `Silly-Response=true` header in them. So you can check for it, and for example, log an error.

#### Usage example
Let's imagine we've started to get tired from all this sillines and want to reduce its amount by calling HTTP method **DELETE** on `https://api.service.example/cats/1`.


To do this, add your destination to the Silly CORS deployment path. In this example, it's `https://silly-cors.deployment/`. And don't forget about trailing slash or you'll end up with an invalid domain!

After awaiting a promise, we make sure that no errors has occured in Silly CORS by checking the header `Silly-Response`.

```js
const url = "https://silly-cors.deployment/" + "https://api.service.example/cats/1";
const response = await fetch(url, {
    method: "DELETE", 
});

if (result.headers.get("Silly-Response")) {
    console.error("Oops! Silliness overload", response, await response.text());
    return;
}

return result.json();
```