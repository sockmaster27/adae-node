# Create an image containing the dependencies required for the CI pipeline

FROM holgermogensen/voil-ci:adae

# Use bash as the default shell
SHELL ["/bin/bash", "-c"]

# Update apt-get index
RUN apt-get update

# Install utilities
RUN apt-get install -y git

# Install Node and NPM
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
# Must use the shell in 'interactive' mode?
SHELL ["/bin/bash", "-c", "-i"]
RUN source ~/.bashrc && nvm install v22.13.1
SHELL ["/bin/bash", "-c"]
ENV PATH="/root/.nvm/versions/node/v22.13.1/bin:$PATH"
