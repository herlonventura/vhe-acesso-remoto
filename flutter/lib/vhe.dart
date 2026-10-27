import 'dart:io';

// Peers reach this computer by its Tailscale address (100.64.0.0/10), not by the RustDesk ID.
Future<String?> vheTailscaleIp() async {
  try {
    final interfaces =
        await NetworkInterface.list(type: InternetAddressType.IPv4);
    for (final interface in interfaces) {
      for (final address in interface.addresses) {
        final bytes = address.rawAddress;
        if (bytes[0] == 100 && bytes[1] >= 64 && bytes[1] < 128) {
          return address.address;
        }
      }
    }
  } catch (_) {}
  return null;
}
