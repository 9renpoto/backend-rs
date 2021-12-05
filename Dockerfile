FROM opensearchproject/opensearch:1.1.0

# https://opensearch.org/docs/latest/opensearch/install/plugins/
RUN bin/opensearch-plugin install analysis-kuromoji

