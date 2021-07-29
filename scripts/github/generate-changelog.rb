# frozen_string_literal: true

require 'base64'
require 'changelogerator'
require 'erb'
require 'git'
require 'json'
require 'octokit'
require 'toml'
require_relative './lib.rb'

current_repository_ref = ENV['GITHUB_REF']
token = ENV['GITHUB_TOKEN']
github_client = Octokit::Client.new(
  access_token: token
)

repo_path = ENV['GITHUB_WORKSPACE'] + '/' + ENV['GITHUB_REPOSITORY'].split('/')[-1]

renderer = ERB.new(
  File.read(__dir__ + '/changelog-template.erb'),
  trim_mode: '<>'
)

latest_release_ref = 'refs/tags/' + github_client.latest_release(ENV['GITHUB_REPOSITORY']).tag_name

current_repository_changelog = Changelog.new(
  ENV['GITHUB_REPOSITORY'], latest_release_ref, current_repository_ref, token: token
)

=begin
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

substrate_previous_sha = get_substrate_commit(github_client, latest_release_ref)
substrate_current_sha = get_substrate_commit(github_client, current_repository_ref)

substrate_repository_changelog = Changelog.new(
  'paritytech/substrate', substrate_previous_sha, substrate_current_sha,
  token: token,
  prefix: true
)

all_changes = (current_repository_changelog.changes + substrate_repository_changelog.changes).reject do |c|
  c[:title] =~ /[Cc]ompanion/
end
=end

all_changes = current_repository_changelog.changes.reject do |c|
  c[:title] =~ /[Cc]ompanion/
end

misc_changes = Changelog.changes_with_label(all_changes, 'B1-releasenotes')
client_changes = Changelog.changes_with_label(all_changes, 'B5-clientnoteworthy')
runtime_changes = Changelog.changes_with_label(all_changes, 'B7-runtimenoteworthy')

# add the audit status for runtime changes
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
manta_pc_runtime = get_runtime('manta-pc', repo_path)
calamari_runtime = get_runtime('calamari', repo_path)

manta_pc_json = JSON.parse(
  File.read(
    "#{ENV['GITHUB_WORKSPACE']}/manta-pc-srtool-output.json"
  )
)

calamari_json = JSON.parse(
  File.read(
    "#{ENV['GITHUB_WORKSPACE']}/calamari-srtool-output.json"
  )
)

puts renderer.result
