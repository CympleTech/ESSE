import 'package:flutter/material.dart';

class InputText extends StatelessWidget {
  final IconData icon;
  final String text;
  final TextEditingController controller;
  final FocusNode focus;

  const InputText({Key? key, required this.icon, required this.text, required this.controller, required this.focus})
  : super(key: key);
  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20.0),
      height: 50.0,
      width: 600.0,
      decoration: BoxDecoration(
        color: color.surface,
        border: Border.all(color: focus.hasFocus ? color.primary : color.surface),
        borderRadius: BorderRadius.circular(15.0)
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: <Widget>[
          Container(
            padding: const EdgeInsets.only(right: 20.0),
            child: Icon(
              icon,
              size: 20.0,
              color: color.primary,
          )),
          Expanded(
            child: TextField(
              style: TextStyle(fontSize: 16.0),
              controller: controller,
              focusNode: focus,
              decoration: InputDecoration(
                hintText: text,
                hintStyle: TextStyle(color: color.onPrimary.withOpacity(0.5)),
                border: InputBorder.none,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
