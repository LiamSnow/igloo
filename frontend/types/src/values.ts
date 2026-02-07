// TODO code generation?

export type IglooValue =
  | { Boolean: boolean }
  | { Real: number }
  | { Integer: number }
  | { Text: string };

export enum IglooType {
  Boolean = "Boolean",
  Real = "Real",
  Integer = "Integer",
  Text = "Text",
}
