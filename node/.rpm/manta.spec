%define debug_package %{nil}

Name: Manta
Summary: Implementation of a https://manta.network node in Rust based on the Substrate framework.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: GPLv3
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

Requires: systemd, shadow-utils
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}


%prep
%setup -q


%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%post
config_file="/etc/default/manta"
getent group manta >/dev/null || \
    groupadd \
        -r manta
getent passwd manta >/dev/null || \
    useradd \
        -r \
        -g manta \
        -d /home/manta \
        -m \
        -s /sbin/nologin \
        -c "service user account for running manta as a service" \
        manta
exit 0

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/etc/default/manta
/usr/lib/systemd/system/manta.service
