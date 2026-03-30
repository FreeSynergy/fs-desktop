FROM fedora:41

# Install Wayland runtime dependencies
RUN dnf install -y \
    wayland-devel \
    mesa-dri-drivers \
    libglvnd-opengl \
    libglvnd-egl \
    fontconfig \
    && dnf clean all

WORKDIR /app

# Copy pre-built binary (built by CI)
COPY target/release/fs-desktop /usr/local/bin/fs-desktop

# Runtime user (must have display access)
RUN useradd -r -s /bin/bash fsdesktop

USER fsdesktop

ENTRYPOINT ["/usr/local/bin/fs-desktop"]

LABEL org.opencontainers.image.source="https://github.com/FreeSynergy/fs-desktop"
LABEL org.opencontainers.image.description="FreeSynergy Desktop Shell"
