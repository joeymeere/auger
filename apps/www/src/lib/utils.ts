import { ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function classNames(...classes: any) {
  return classes.filter(Boolean).join(" ");
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
