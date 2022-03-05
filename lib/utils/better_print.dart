String pidText(String? pid, [String pre='EH']) {
  if (pid == null) {
    return '';
  }
  return pre + pid.toUpperCase();
}

String pidPrint(String? pid, [String pre='EH', int n = 6]) {
  if (pid == null) {
    return '';
  }

  final info = pid.toUpperCase();
  final len = info.length;
  if (len > n+n) {
    return pre + info.substring(0, n) + '...' + info.substring(len - n, len);
  } else {
    return info;
  }
}

String pidParse(String pid, [String pre='EH']) {
  if (pid.length > 2 && pid.substring(0, 2) == pre) {
    return pid.substring(2);
  } else {
    return pid;
  }
}

String addrText(String? addr) {
  if (addr == null) {
    return '';
  }
  return '0x' + addr.toLowerCase();
}

String addrPrint(String? addr) {
  if (addr == null) {
    return '';
  }

  final info = addr.toLowerCase();
  final len = info.length;
  if (len > 12) {
    return '0x' + info.substring(0, 6) + '...' + info.substring(len - 6, len);
  } else {
    return info;
  }
}

String addrParse(String addr) {
  if (addr.length > 2 && addr.substring(0, 2) == '0x') {
    return addr.substring(2);
  } else {
    return addr;
  }
}
