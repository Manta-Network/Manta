%define debug_package %{nil}

Name: Manta
Summary: https://manta.network and https://calamari.network substrate service nodes
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
ln -sf /usr/bin/manta %{buildroot}/usr/bin/calamari

%post
config_file="/etc/default/manta"
getent group manta >/dev/null || \
    groupadd \
        --system \
        manta
getent passwd manta >/dev/null || \
    useradd \
        --system \
        --gid manta \
        --home-dir /var/lib/substrate \
        --create-home \
        --shell /sbin/nologin \
        --comment "service account for manta and calamari services" \
        manta
exit 0

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/etc/default/calamari
/etc/default/manta
/usr/lib/systemd/system/calamari.service
/usr/lib/systemd/system/manta.service
/usr/share/substrate/calamari.json
/usr/share/substrate/kusama.json
/usr/share/substrate/manta.json
/usr/share/substrate/polkadot.json