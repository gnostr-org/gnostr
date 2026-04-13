
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/build-artifact.yml"); puts "YAML OK"' && git --no-pager diff -- .github/workflows/build-artifact.yml
