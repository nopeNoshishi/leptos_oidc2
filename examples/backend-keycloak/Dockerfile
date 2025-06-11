ARG KEYCLOAK_VERSION=26.1.0

FROM registry.access.redhat.com/ubi9 AS ubi-micro-build

RUN mkdir -p /mnt/rootfs
# Removes this warning "This system is not registered to Red Hat Subscription Management. You can use subscription-manager to register."
RUN dnf remove -y subscription-manager
RUN dnf install --installroot /mnt/rootfs curl jq --releasever 9 --setopt install_weak_deps=false --nodocs -y; dnf --installroot /mnt/rootfs clean all

# Documentation:
# - HowTo add curl to the keycloak image: https://github.com/keycloak/keycloak/issues/17438
# - Creating a customized and optimized container image: https://www.keycloak.org/server/containers

FROM quay.io/keycloak/keycloak:$KEYCLOAK_VERSION
USER root

COPY --from=ubi-micro-build /mnt/rootfs/usr/bin/curl /usr/bin/curl
COPY --from=ubi-micro-build /mnt/rootfs/usr/bin/jq /usr/bin/jq
COPY --from=ubi-micro-build /mnt/rootfs/usr/lib64/ /usr/lib64/

COPY ./keycloak_functions.sh /keycloak_functions.sh
COPY ./provision.sh /provision.sh

RUN /opt/keycloak/bin/kc.sh build

USER keycloak
