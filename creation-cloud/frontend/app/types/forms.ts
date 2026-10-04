export interface FormField {
  key: string;
  label: [string, string];
  kind?:
    | "text"
    | "email"
    | "password"
    | "number"
    | "textarea"
    | "checkbox"
    | "select";
  options?: { value: string; label: [string, string] }[];
  required?: boolean;
  createOnly?: boolean;
  editOnly?: boolean;
  min?: number;
  max?: number;
  default?: string | number | boolean;
}
export interface ResourceConfig {
  title: [string, string];
  endpoint: string;
  fields: FormField[];
  columns: { key: string; label: [string, string] }[];
  search?: boolean;
  array?: boolean;
  revision?: boolean;
}

export type ContentNode =
  | string
  | number
  | boolean
  | null
  | ContentNode[]
  | { [key: string]: ContentNode };
