# KeyCloak

This guide should help you setting up **leptos_oidc** with [KeyCloak](https://github.com/sebadob/rauthy).

## Setup your KeyCloak

You can find a guide to [setup KeyCloak](https://www.keycloak.org/getting-started/getting-started-docker),
all you need to do is setting up a realm. The client part will be explained here.

## Setup KeyCloak

Setting up KeyCloak is quite easily, all you need to do is creating a new
client. In this example we will call that client localdev. \
![add client in keycloak](keycloak_create_client.png){width=50%}

This step is optional, you can disable the `direct access grant`, but you don't
need to do it when you are working with **leptos_oidc** \
![disable direct access grants in keycloak](keycloak_disable_direct_access_grants.png){width=50%}

After creating the new client, all you need to do is adding the `Valid redirect
URIs`, `Valid post logut redirect URIs` and `Web origins`. Please keep in mind
that you need to add the `?destroy_session=true` at the end of the domain, this
will be added automatically from the **leptos_oidc** library and is mandatory.
And of course, don't forget to save at the end. \
![enable refresh token response in rauthy](keycloak_add_urls.png){width=50%}

## Setup leptos_oidc

All you need to do is to set up everything with the init function. In this
example the config would look like this:
[auth](../../examples/simple/src/simple)

Please keep in mind to set your realm correctly, otherwise it won't work.
