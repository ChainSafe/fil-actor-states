# frozen_string_literal: true

require 'optparse'
require 'toml-rb'
require 'set'

options = { ignore: Set.new }
OptionParser.new do |opts|
  opts.on('--ignore=IGNORE', 'Ignore package') do |v|
    options[:ignore].add v
  end
end.parse!

exit_code = 0

def get_pattern(crate_raw)
  crate = crate_raw.gsub(/-/, '_')
  Regexp.new("(\\buse\\s#{crate}\\b)|(\\b#{crate}::)")
end

def extract_and_add(crates, toml, key)
  toml[key]&.each do |crate_name, _|
    crates.add crate_name
  end
end

def extract_crates(toml)
  crates = Set.new
  extract_and_add(crates, toml, 'dependencies')
  extract_and_add(crates, toml, 'dev-dependencies')
  extract_and_add(crates, toml, 'build-dependencies')
  crates
end

Dir.glob('**/*.toml').each do |file|
  crate_dir = File.dirname(file)
  toml = TomlRB.load_file(file)
  crates = extract_crates toml
  crates.each do |crate|
    break if options[:ignore].include? crate

    used = false
    pattern = get_pattern(crate)
    Dir.glob("#{crate_dir}/**/*.rs").each do |rs|
      used |= File.read(rs).match?(pattern)
    end
    unless used
      puts "Protentially unused: #{crate} in #{crate_dir}"
      exit_code = 1
    end
  end
end

exit exit_code
