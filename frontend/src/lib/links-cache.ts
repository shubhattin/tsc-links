/** Keep the `['links']` query in sync after mutations without a refetch round-trip. */
import { LINKS_QUERY_KEY, type LinkDto } from '$lib/links-api';
import { queryClient } from '$lib/query-client';

export function addLinkToCache(link: LinkDto) {
  queryClient.setQueryData<LinkDto[]>(LINKS_QUERY_KEY, (old) =>
    old ? [link, ...old.filter((l) => l.id !== link.id)] : old,
  );
}

export function replaceLinkInCache(link: LinkDto) {
  queryClient.setQueryData<LinkDto[]>(LINKS_QUERY_KEY, (old) =>
    old?.map((l) => (l.id === link.id ? link : l)),
  );
}

export function removeLinkFromCache(id: string) {
  queryClient.setQueryData<LinkDto[]>(LINKS_QUERY_KEY, (old) => old?.filter((l) => l.id !== id));
}
