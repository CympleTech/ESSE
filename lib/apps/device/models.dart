import 'package:esse/utils/relative_time.dart';

class Device {
  int id;
  String name;
  String info;
  String addr;
  RelativeTime lastTime;
  bool online = false;

  Device.fromList(List params):
    this.id = params[0],
    this.name = params[1],
    this.info = params[2],
    this.addr = params[3],
    this.lastTime = RelativeTime.fromInt(params[4]),
    this.online = params[5] == "1";

  String printAddr() {
    return '0x' + this.addr.substring(0, 6) + "..." + this.addr.substring(64 - 8, 64);
  }
}

class DeviceStatus {
  int cpu = 0;
  int cpuUsed = 0;
  int memory = 0;
  int memoryUsed = 0;
  int swap = 0;
  int swapUsed = 0;
  int disk = 0;
  int diskUsed = 0;
  RelativeTime uptime = RelativeTime();

  DeviceStatus();

  DeviceStatus.fromList(List params) {
    this.cpu = params[0];
    this.memory = params[1];
    this.swap = params[2];
    this.disk = params[3];

    this.cpuUsed = params[4];
    this.memoryUsed = params[5];
    this.swapUsed = params[6];
    this.diskUsed = params[7];

    this.uptime = RelativeTime.fromInt(params[8]);
  }

  static String format(int n) {
    if (n >= 1024) {
      final m = (n/1024).toStringAsFixed(2);
      return "${m} GB";
    } else {
      return "${n}.00 MB";
    }
  }

  String cpu_u() {
    return "${this.cpu}";
  }

  String memory_u() {
    return format(this.memory);
  }

  String swap_u() {
    return format(this.swap);
  }

  String disk_u() {
    return format(this.disk);
  }

  double cpu_p() {
    final p = this.cpuUsed/100;
    return p < 100 ? p : 100;
  }

  double memory_p() {
    final p = this.memoryUsed/100;
    return p < 100 ? p : 100;
  }

  double swap_p() {
    final p = this.swapUsed/100;
    return p < 100 ? p : 100;
  }

  double disk_p() {
    final p = this.diskUsed/100;
    return p < 100 ? p : 100;
  }
}
