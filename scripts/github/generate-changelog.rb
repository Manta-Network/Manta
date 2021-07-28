# frozen_string_literal: true

require 'base64'
require 'changelogerator'
require 'erb'
require 'git'
require 'json'
require 'octokit'
require 'toml'
require_relative './lib.rb'

current_ref = ENV['GITHUB_REF']
token = ENV['GITHUB_TOKEN']
github_client = Octokit::Client.new(
  access_token: token
)

repo_path = ENV['GITHUB_WORKSPACE'] + '/Manta/'

# Generate an ERB renderer based on the template .erb file
renderer = ERB.new(
  File.read(ENV['GITHUB_WORKSPACE'] + '/Manta/scripts/github/manta-release.erb'),
  trim_mode: '<>'
)

# get ref of last release
last_ref = 'refs/tags/' + github_client.latest_release(ENV['GITHUB_REPOSITORY']).tag_name

manta_cl = Changelog.new(
  'Manta-Network/Manta', last_ref, current_ref, token: token
)

# gets the substrate commit hash used for a given ref
def get_substrate_commit(client, ref)
  cargo = TOML::Parser.new(
    Base64.decode64(
      client.contents(
        ENV['GITHUB_REPOSITORY'],
        path: 'Cargo.lock',
        query: { ref: ref.to_s }
      ).content
    )
  ).parsed
  cargo['package'].find { |p| p['name'] == 'sc-cli' }['source'].split('#').last
end

substrate_prev_sha = get_substrate_commit(github_client, last_ref)
substrate_cur_sha = get_substrate_commit(github_client, current_ref)

substrate_cl = Changelog.new(
  'paritytech/substrate', substrate_prev_sha, substrate_cur_sha,
  token: token,
  prefix: true
)

all_changes = (manta_cl.changes + substrate_cl.changes).reject do |c|
  c[:title] =~ /[Cc]ompanion/
end

misc_changes = Changelog.changes_with_label(all_changes, 'B1-releasenotes')
client_changes = Changelog.changes_with_label(all_changes, 'B5-clientnoteworthy')
runtime_changes = Changelog.changes_with_label(all_changes, 'B7-runtimenoteworthy')

# Add the audit status for runtime changes
runtime_changes.each do |c|
  if c[:labels].any? { |l| l[:name] == 'D1-audited 👍' }
    c[:pretty_title] = "✅ `audited` #{c[:pretty_title]}"
    next
  end
  if c[:labels].any? { |l| l[:name] == 'D2-notlive 💤' }
    c[:pretty_title] = "✅ `not live` #{c[:pretty_title]}"
    next
  end
  if c[:labels].any? { |l| l[:name] == 'D3-trivial 🧸' }
    c[:pretty_title] = "✅ `trivial` #{c[:pretty_title]}"
    next
  end
  if c[:labels].any? { |l| l[:name] == 'D5-nicetohaveaudit ⚠️' }
    c[:pretty_title] = "⏳ `pending non-critical audit` #{c[:pretty_title]}"
    next
  end
  if c[:labels].any? { |l| l[:name] == 'D9-needsaudit 👮' }
    c[:pretty_title] = "❌ `AWAITING AUDIT` #{c[:pretty_title]}"
    next
  end
  c[:pretty_title] = "⭕️ `unknown audit requirements` #{c[:pretty_title]}"
end

release_priority = Changelog.highest_priority_for_changes(client_changes)

rustc_stable = ENV['RUSTC_STABLE']
rustc_nightly = ENV['RUSTC_NIGHTLY']
manta_pc_runtime = get_runtime('manta_pc', repo_path)
calamari_runtime = get_runtime('calamari', repo_path)

manta_pc_json = JSON.parse(
  File.read(
    "#{ENV['GITHUB_WORKSPACE']}/manta_pc-srtool-json/manta_pc_srtool_output.json"
  )
)

calamari_json = JSON.parse(
  File.read(
    "#{ENV['GITHUB_WORKSPACE']}/calamari-srtool-json/calamari_srtool_output.json"
  )
)

puts renderer.result
