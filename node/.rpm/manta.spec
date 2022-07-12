%define debug_package %{nil}

Name: manta
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

%pre
if [ $1 -eq 2 ] ; then
    if systemctl is-active --quiet calamari.service; then
        systemctl stop calamari.service
    fi
    if systemctl is-active --quiet manta.service; then
        systemctl stop manta.service
    fi
fi
if ! getent group manta > /dev/null 2>&1; then
    groupadd --system manta
    echo "groupadd manta, result: $?"
fi
if ! getent passwd manta > /dev/null 2>&1; then
    useradd \
        --system \
        --gid manta \
        --home-dir /var/lib/substrate \
        --create-home \
        --shell /sbin/nologin \
        --comment "service account for manta and calamari services" \
        manta
    echo "useradd manta, result: $?"
fi
exit 0

%post
if ! getent group manta > /dev/null 2>&1; then
    groupadd --system manta
    echo "groupadd manta, result: $?"
fi
if ! getent passwd manta > /dev/null 2>&1; then
    useradd \
        --system \
        --gid manta \
        --home-dir /var/lib/substrate \
        --create-home \
        --shell /sbin/nologin \
        --comment "service account for manta and calamari services" \
        manta
    echo "useradd manta, result: $?"
fi
exit 0

%preun
if systemctl is-active --quiet calamari.service; then
    systemctl stop calamari.service
fi
if systemctl is-active --quiet manta.service; then
    systemctl stop manta.service
fi
if systemctl is-enabled --quiet calamari.service; then
    systemctl disable calamari.service
fi
if systemctl is-enabled --quiet manta.service; then
    systemctl disable manta.service
fi
exit 0

%postun
if [ $1 -eq 0 ] ; then
    if getent passwd manta > /dev/null 2>&1; then
        userdel --remove manta
        echo "userdel manta, result: $?"
    fi
    if getent group manta > /dev/null 2>&1; then
        groupdel manta
        echo "groupdel manta, result: $?"
    fi
fi
exit 0

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/usr/lib/systemd/system/calamari.service
/usr/lib/systemd/system/dolphin.service
/usr/lib/systemd/system/manta.service
/usr/share/substrate/calamari.json
/usr/share/substrate/kusama.json
/usr/share/substrate/manta.json
/usr/share/substrate/polkadot.json
/usr/share/substrate/rococo.json