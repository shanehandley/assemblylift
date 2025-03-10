#!/usr/bin/ruby

#
# The AssemblyLift build script
#

require 'shellwords'


# Check environment
# Need: docker, cargo, rustc
def check_exists(executable)
  `#{executable} &> /dev/null`

  if $?.exitstatus != 0
    puts "Could not exec #{executable}"
    false
  else
    puts "Found #{executable}!"
    true
  end
end

def die(message)
  puts "DIE: " + message
  exit(-1)
end

env_has_docker = check_exists("docker")
env_has_cargo = check_exists("cargo")
env_has_rustc = check_exists("rustc")
env_has_cmake = check_exists("cmake") # needed for some dependencies

DOCKER = "docker"
CARGO = "cargo"
RUSTC = "rustc"


# Check that a command was given
args = %w[build test]
arg_error_string = "build.rb must be run with one of #{args} as an argument"

unless ARGV[0]
  die(arg_error_string)
end

cmd = ARGV[0]

unless args.include?(cmd)
  die(arg_error_string)
end


# Switch on commands
case cmd
when "build"
  build_args = %w[local deploy]
  build_arg_error_string = "build.rb build command must be run with one of #{build_args} as an argument"

  build_cmd = ARGV[1]

  unless build_args.include?(build_cmd)
    die(build_arg_error_string)
  end

  case build_cmd
  when "local"
    unless env_has_cargo and env_has_rustc and env_has_cmake
      die("Missing build dependency, exiting...")
    end

    puts "Building local build..."
    super_args = ARGV[2..ARGV.length].map{|arg| Shellwords.escape arg}.join(' ')
    `#{CARGO} build #{super_args}`

  when "deploy"
    unless env_has_docker
      die("Missing docker, exiting...")
    end

    version = "0.1.0" # TODO load from cli/Cargo.toml
    tag = "assemblylift:#{version}"

    puts "Building deployment-ready build..."
    `#{DOCKER} build . --file Dockerfile_aws-lambda --tag #{tag}`
    `#{DOCKER} run --rm --entrypoint cat #{tag} /usr/src/assemblylift/target/release/bootstrap > ./bootstrap`
    `#{DOCKER} run --rm --entrypoint cat #{tag} /usr/src/assemblylift/target/release/libassemblylift_awslambda_iomod_plugin_dynamodb.so > ./libassemblylift_awslambda_iomod_plugin_dynamodb.so`
    `chmod 777 ./bootstrap`
    `zip ./bootstrap.zip ./bootstrap`
    puts "Done! Build artifacts copied to project root."

  else
    die(build_arg_error_string)
  end

when "test"
  die("test is not yet implemented")

else
  die(arg_error_string)
end
