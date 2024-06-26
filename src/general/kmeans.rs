// Macro to implement kmeans for both f64 and f32 without writing everything
// twice or importing the `num` crate
macro_rules! impl_kmeans {
    ($kind: ident) => {
        // Since we can't overload methods in rust, we have to use namespace
        pub mod $kind {
            use std::$kind::INFINITY;
            /// computes sum of squared deviation between two identically sized vectors
            /// `x`, and `y`.
            fn distance(x: &[$kind], y: &[$kind]) -> $kind {
                x.iter()
                    .zip(y.iter())
                    .fold(0.0, |dist, (&xi, &yi)| dist + (xi - yi).powi(2))
            }

            /// Returns a vector containing the indices z<sub>i</sub> in {0, ..., K-1} of
            /// the centroid nearest to each datum.
            fn nearest_centroids(xs: &[Vec<$kind>], centroids: &[Vec<$kind>]) -> Vec<usize> {
                xs.iter()
                    .map(|xi| {
                        // Find the argmin by folding using a tuple containing the argmin
                        // and the minimum distance.
                        let (argmin, _) = centroids.iter().enumerate().fold(
                            (0_usize, INFINITY),
                            |(min_ix, min_dist), (ix, ci)| {
                                let dist = distance(xi, ci);
                                if dist < min_dist {
                                    (ix, dist)
                                } else {
                                    (min_ix, min_dist)
                                }
                            },
                        );
                        argmin
                    })
                    .collect()
            }

            /// Recompute the centroids given the current clustering
            fn recompute_centroids(
                xs: &[Vec<$kind>],
                clustering: &[usize],
                k: usize,
            ) -> Vec<Vec<$kind>> {
                let ndims = xs[0].len();

                // NOTE: Kind of inefficient because we sweep all the data from each of the
                // k centroids.
                (0..k)
                    .map(|cluster_ix| {
                        let mut centroid: Vec<$kind> = vec![0.0; ndims];
                        let mut n_cluster: $kind = 0.0;
                        xs.iter().zip(clustering.iter()).for_each(|(xi, &zi)| {
                            if zi == cluster_ix {
                                n_cluster += 1.0;
                                xi.iter().enumerate().for_each(|(j, &x_ij)| {
                                    centroid[j] += x_ij;
                                });
                            }
                        });
                        centroid.iter().map(|&c_j| c_j / n_cluster).collect()
                    })
                    .collect()
            }

            /// Assign the N D-dimensional data, `xs`, to `k` clusters using
            /// K-Means clustering, with optional iteration limitation `max_iter`
            pub fn kmeans(
                xs: &Vec<Vec<$kind>>,
                k: usize,
                max_iter: Option<i32>,
            ) -> Option<Vec<usize>> {
                if xs.len() < k || k == 0 {
                    return None;
                }

                // Rather than pulling in a dependency to randomly select the staring
                // points for the centroids, we're going to deterministically choose them by
                // selecting evenly spaced points in `xs`
                let n_per_cluster: usize = xs.len() / k;
                let centroids: Vec<Vec<$kind>> =
                    (0..k).map(|j| xs[j * n_per_cluster].clone()).collect();

                let mut clustering = nearest_centroids(&xs, &centroids);

                let mut count_iter: i32 = 0;

                while max_iter == None || count_iter < max_iter.unwrap() {
                    let centroids = recompute_centroids(&xs, &clustering, k);
                    let new_clustering = nearest_centroids(&xs, &centroids);

                    // loop until the clustering doesn't change after the new centroids are computed
                    if new_clustering
                        .iter()
                        .zip(clustering.iter())
                        .all(|(&za, &zb)| za == zb)
                    {
                        // break loop and return since the result converges
                        break;
                    } else {
                        clustering = new_clustering;
                    }

                    count_iter += 1;
                }

                Some(clustering)
            }
        }
    };
}

// generate code for kmeans for f32 and f64 data
impl_kmeans!(f64);
impl_kmeans!(f32);

#[cfg(test)]
mod test {
    use self::super::f64::kmeans;
    use crate::machine_learning::k_means;
    use rand::random;

    #[test]
    fn easy_univariate_clustering() {
        let xs: Vec<Vec<f64>> = vec![
            vec![-1.1],
            vec![-1.2],
            vec![-1.3],
            vec![-1.4],
            vec![1.1],
            vec![1.2],
            vec![1.3],
            vec![1.4],
        ];
        let clustering = kmeans(&xs, 2, None);
        assert_eq!(clustering.unwrap(), vec![0, 0, 0, 0, 1, 1, 1, 1]);
    }

    #[test]
    fn easy_univariate_clustering_odd_number_of_data() {
        let xs: Vec<Vec<f64>> = vec![
            vec![-1.1],
            vec![-1.2],
            vec![-1.3],
            vec![-1.4],
            vec![1.1],
            vec![1.2],
            vec![1.3],
            vec![1.4],
            vec![1.5],
        ];
        let clustering = kmeans(&xs, 2, None);
        assert_eq!(clustering.unwrap(), vec![0, 0, 0, 0, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn easy_bivariate_clustering() {
        let xs: Vec<Vec<f64>> = vec![
            vec![-1.1, 0.2],
            vec![-1.2, 0.3],
            vec![-1.3, 0.1],
            vec![-1.4, 0.4],
            vec![1.1, -1.1],
            vec![1.2, -1.0],
            vec![1.3, -1.2],
            vec![1.4, -1.3],
        ];
        let clustering = kmeans(&xs, 2, None);
        assert_eq!(clustering.unwrap(), vec![0, 0, 0, 0, 1, 1, 1, 1]);
    }

    #[test]
    fn high_dims() {
        let xs: Vec<Vec<f64>> = vec![
            vec![-2.7825343, -1.7604825, -5.5550113, -2.9752946, -2.7874138],
            vec![-2.9847919, -3.8209332, -2.1531757, -2.2710119, -2.3582877],
            vec![-3.0109320, -2.2366132, -2.8048492, -1.2632331, -4.5755581],
            vec![-2.8432186, -1.0383805, -2.2022826, -2.7435962, -2.0013399],
            vec![-2.6638082, -3.5520086, -1.3684702, -2.1562444, -1.3186447],
            vec![1.7409171, 1.9687576, 4.7162628, 4.5743537, 3.7905611],
            vec![3.2932369, 2.8508700, 2.5580937, 2.0437325, 4.2192562],
            vec![2.5843321, 2.8329818, 2.1329531, 3.2562319, 2.4878733],
            vec![2.1859638, 3.2880048, 3.7018615, 2.3641232, 1.6281994],
            vec![2.6201773, 0.9006588, 2.6774097, 1.8188620, 1.6076493],
        ];

        let clustering = kmeans(&xs, 2, None);
        assert_eq!(clustering.unwrap(), vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_edge_cases() {
        let xs = vec![];
        let clustering = kmeans(&xs, 0, None);
        assert_eq!(clustering, None);
        let clustering = kmeans(&xs, 1234, None);
        assert_eq!(clustering, None);
        let xs = vec![vec![1.0], vec![2.0], vec![3.0]];
        let clustering = kmeans(&xs, 4, None);
        assert_eq!(clustering, None);
    }

    /// This test eventually shows that we don't need
    /// two implementation of k means. `general::kmeans_XXX`
    /// may the better one according to its generalization on
    /// **dimension** and **type** compare to `machine_learning::k_means`.
    #[test]
    fn compare_two_impl_of_k_means() {
        let mut xs: Vec<Vec<f64>> = vec![];
        let mut data_points: Vec<(f64, f64)> = vec![];
        let n_points: usize = 1000;

        for _ in 0..n_points {
            let x: f64 = random::<f64>() * 100.0;
            let y: f64 = random::<f64>() * 100.0;

            xs.push(vec![x, y]);
            data_points.push((x, y));
        }

        let max_iter = 100;

        assert_eq!(
            kmeans(&xs, 10, Some(max_iter)),
            k_means(data_points, 10, max_iter)
        );
    }
}
