import { createPrinciSocialImage, socialImageContentType, socialImageSize } from "@/components/site-social-image";

export const size = socialImageSize;
export const contentType = socialImageContentType;
export const alt = "PrinciPL v0.1, a statically typed language for native Windows x86-64 programs";

export default function OpenGraphImage() {
  return createPrinciSocialImage();
}
