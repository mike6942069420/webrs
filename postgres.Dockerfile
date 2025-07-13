FROM postgres:latest

# Change UID and GID of the postgres user to 1000:1000
RUN usermod -u 1000 postgres && \
    groupmod -g 1000 postgres && \
    chown -R postgres:postgres /var/lib/postgresql

# Ensure ownership of data directory
RUN chown -R postgres:postgres /var/lib/postgresql/data

USER postgres
