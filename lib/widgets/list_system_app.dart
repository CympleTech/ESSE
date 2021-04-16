import 'package:flutter/material.dart';

class ListSystemApp extends StatelessWidget {
  final String name;
  final IconData icon;
  final Function callback;

  const ListSystemApp({Key key, this.name, this.icon, this.callback}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: callback,
      child: SizedBox(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(left: 20.0, right: 15.0),
              child: Icon(icon, color: color.primary, size: 24.0),
              decoration: BoxDecoration(
                color: color.surface,
                borderRadius: BorderRadius.circular(15.0)
              ),
            ),
            Text(name, style: TextStyle(fontSize: 16.0)),
          ],
        ),
      ),
    );
  }
}
