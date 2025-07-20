import type { ComponentType, ReactNode } from "react";

// biome-ignore lint/suspicious/noExplicitAny: explanation
type Provider<P = any> = [ComponentType<P>, P?];

export const composeProviders = (providers: Provider[]): ComponentType<{ children: ReactNode }> => {
  return providers.reduceRight(
    (AccumulatedProviders, [Provider, props = {}]) => {
      return ({ children }: { children: ReactNode }) => (
        <Provider {...props}>
          <AccumulatedProviders>{children}</AccumulatedProviders>
        </Provider>
      );
    },
    ({ children }: { children: ReactNode }) => <>{children}</>,
  );
};
