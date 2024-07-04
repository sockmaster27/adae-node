# Create an image containing the dependencies required for the CI pipeline

FROM holgermogensen/voil-ci:adae

# Use bash as the default shell
SHELL ["/bin/bash", "-c"]

# Update apt-get index
RUN apt-get update

# Install utilities
RUN apt-get install -y git

# Install Bun
RUN apt-get install -y unzip
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"
